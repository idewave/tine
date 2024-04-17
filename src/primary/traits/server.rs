use std::sync::{Arc};
use std::time::Duration;
use anyhow::{Result as AnyResult};
use async_trait::async_trait;
use colored::Colorize;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
use crate::primary::crypto::header_crypt::HeaderCrypt;

use crate::primary::crypto::srp::Srp;
use crate::primary::server::types::Packet;
use crate::primary::types::{HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult};

pub struct RunOptions {
    pub srp: Arc<Mutex<Srp>>,
}

#[derive(Default, Debug)]
pub struct Connection {
    pub header_crypt: Option<HeaderCrypt>,
}

#[async_trait]
pub trait Server: Send {
    fn new() -> Self;

    async fn run(&mut self, options: Arc<RunOptions>) -> AnyResult<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let listener = TcpListener::bind(format!("{}:{}", Self::host(), Self::port())).await?;
        println!("[{}] is started on port {}", Self::server_name(), Self::port().to_string());

        loop {
            tokio::select! {
                stream = listener.accept() => {
                    match stream {
                        Ok((mut socket, _)) => {
                            let peer_addr = socket.peer_addr().unwrap();
                            {
                                let message = format!("Client connected: {:?}", peer_addr);
                                println!("{}", message.yellow());
                            }

                            let shutdown_tx = shutdown_tx.clone();
                            let options = options.clone();

                            self.init(&mut socket, options.clone()).await;
                            // let connection: Option<Connection> = self.init_connection();

                            tokio::spawn(async move {
                                if let Err(err) = Self::handle_connection(socket, options).await {
                                    eprintln!(
                                        "[{}]: Error handling connection: {}",
                                        Self::server_name(),
                                        err
                                    );
                                }

                                drop(shutdown_tx);
                            });
                        },
                        Err(err) => {
                            eprintln!("Error accepting connection: {}", err);
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn read_packet(
        socket: &mut TcpStream,
        connection: Arc<Mutex<Connection>>
    ) -> AnyResult<Packet>;

    async fn handle_connection(mut socket: TcpStream, options: Arc<RunOptions>) -> AnyResult<()> {
        let connection = Arc::new(Mutex::new(Connection::default()));

        loop {
            match Self::read_packet(&mut socket, connection.clone()).await {
                Ok(packet) => {
                    let Packet { data, opcode } = packet;
                    println!("RECEIVED: {:?}", Opcode::get_opcode_name(opcode));

                    let mut input = HandlerInput {
                        data,
                        opcode,
                        srp: Arc::clone(&options.srp),
                        connection: Arc::clone(&connection),
                    };

                    let handler_list = Self::get_processors()
                        .iter()
                        .flat_map(|processor| processor(&mut input))
                        .collect::<ProcessorResult>();

                    for mut handler in handler_list {
                        let response = handler.handle(&mut input).await;
                        match response {
                            Ok(outputs) => {
                                for output in outputs {
                                    match output {
                                        HandlerOutput::Data(mut packet) => {
                                            let mut guard = connection.lock().await;
                                            if let Some(mut header_crypt) = guard.header_crypt.as_mut() {
                                                let is_large_packet = packet[0] >= 0x80;
                                                let header_size: usize = if is_large_packet { 5 } else { 4 };

                                                let encrypted_header = header_crypt
                                                    .encrypt(&packet[..header_size].to_vec());
                                                packet[..header_size].copy_from_slice(&encrypted_header);
                                            }

                                            socket.write_all(&packet).await.unwrap();
                                        },
                                        HandlerOutput::SessionKey(_key) => {},
                                    }
                                }
                            },
                            Err(err) => {
                                println!("[ERROR]: {}", err.to_string().red())
                            },
                        };
                    }
                },
                Err(err) => {
                    println!("Error on packet read: {:?}", err.to_string());
                    break;
                },
            }

            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn init(&mut self, _socket: &mut TcpStream, _options: Arc<RunOptions>) {
        // do nothing by default, but can contain some preparation steps
    }

    fn get_processors() -> Vec<ProcessorFunction>;

    fn host<'a>() -> &'a str;

    fn port() -> u16;

    fn server_name<'a>() -> &'a str;
}