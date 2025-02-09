use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender};
use async_trait::async_trait;
use colored::Colorize;
use tentacli_traits::types::{HandlerOutput as ClientHandlerOutput, opcodes::Opcode};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::{mpsc, Mutex};

use crate::primary::crypto::header_crypt::HeaderCrypt;
use crate::primary::crypto::srp::Srp;
use crate::primary::server::types::Packet;
use crate::primary::types::{
    HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult, ServerInfo,
};

const BUFFER_SIZE: usize = 50;

#[derive(Default)]
pub struct RunOptions {
    pub srp: Arc<Mutex<Srp>>,
    pub sender: Option<BroadcastSender<ClientHandlerOutput>>,
    pub receiver: Option<BroadcastReceiver<ClientHandlerOutput>>,
    pub login_port: u16,
    pub world_port: u16,
}

#[derive(Default, Debug)]
pub struct Connection {
    pub header_crypt: Option<HeaderCrypt>,
}

#[async_trait]
pub trait BaseServer: Send {
    async fn run(&mut self, options: Arc<RunOptions>) -> AnyResult<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let port = Self::port(options.clone());
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        crate::debug!("[{}] is started on port {}", Self::server_name(), port);

        loop {
            tokio::select! {
                stream = listener.accept() => {
                    match stream {
                        Ok((mut socket, _)) => {
                            let peer_addr = socket.peer_addr().unwrap();
                            {
                                let message = format!("Client connected: {:?}", peer_addr);
                                crate::debug!("{}", message.yellow());
                            }

                            let shutdown_tx = shutdown_tx.clone();
                            let options = options.clone();

                            self.init(&mut socket, options.clone()).await;

                            tokio::spawn(async move {
                                if let Err(err) = Self::handle_connection(socket, options).await {
                                    crate::debug!(
                                        "[{}]: Error handling connection: {}",
                                        Self::server_name(),
                                        err
                                    );
                                }

                                drop(shutdown_tx);
                            });
                        },
                        Err(err) => {
                            crate::debug!("Error accepting connection: {}", err);
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
        socket: &mut OwnedReadHalf,
        connection: Arc<Mutex<Connection>>,
    ) -> AnyResult<Packet>;

    async fn handle_connection(mut socket: TcpStream, options: Arc<RunOptions>) -> AnyResult<()> {
        let connection = Arc::new(Mutex::new(Connection::default()));
        let output_receiver = options.receiver.clone();
        let output_sender = options.sender.clone();

        let (mut rx, mut tx) = socket.into_split();
        let (packet_sender, mut packet_receiver) = mpsc::channel::<(u16, Vec<u8>)>(BUFFER_SIZE);

        let mut tasks = vec![];

        if let (Some(mut receiver), Some(sender)) = (output_receiver, output_sender) {
            // the task to handle external packets from tentacli
            let packet_sender = packet_sender.clone();
            tasks.push(tokio::spawn(async move {
                loop {
                    let result = receiver.recv().await;
                    match result {
                        Ok(output) => match output {
                            ClientHandlerOutput::Data((opcode, packet, _)) => {
                                packet_sender.send((opcode as u16, packet)).await.unwrap();
                            }
                            _ => {}
                        },
                        Err(err) => {
                            sender
                                .broadcast(ClientHandlerOutput::ErrorMessage(
                                    "Server cannot process the packet".to_string(),
                                    Some(err.to_string()),
                                ))
                                .await
                                .unwrap();
                        }
                    }
                }
            }));
        }

        {
            // the task to handle local packets
            let connection = connection.clone();
            tasks.push(tokio::spawn(async move {
                let packet_sender = packet_sender.clone();
                loop {
                    match Self::read_packet(&mut rx, connection.clone()).await {
                        Ok(packet) => {
                            let Packet { data, opcode } = packet;
                            crate::debug!(
                                "[RECV]: {}",
                                Opcode::get_opcode_name(opcode).unwrap().blue()
                            );

                            let mut input = HandlerInput {
                                data,
                                opcode,
                                srp: Arc::clone(&options.srp),
                                connection: Arc::clone(&connection),
                                server_info: ServerInfo {
                                    login_port: options.login_port,
                                    world_port: options.world_port,
                                },
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
                                                HandlerOutput::Data(opcode, packet) => {
                                                    packet_sender
                                                        .send((opcode, packet))
                                                        .await
                                                        .unwrap();
                                                }
                                                HandlerOutput::SessionKey(_key) => {}
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        crate::debug!("[ERROR]: {}", err.to_string().red())
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            crate::debug!("Error on packet read: {:?}", err.to_string());
                            break;
                        }
                    }
                }
            }));
        }

        loop {
            if let Some((opcode, mut packet)) = packet_receiver.recv().await {
                let mut guard = connection.lock().await;
                if let Some(header_crypt) = guard.header_crypt.as_mut() {
                    let is_large_packet = packet[0] >= 0x80;
                    let header_size: usize = if is_large_packet { 5 } else { 4 };

                    let encrypted_header = header_crypt.encrypt(&packet[..header_size]);
                    packet[..header_size].copy_from_slice(&encrypted_header);
                }

                crate::debug!(
                    "[SENT]: {}",
                    Opcode::get_opcode_name(opcode as u32).unwrap().magenta()
                );

                tx.write_all(&packet).await.unwrap();
            }
        }

        Ok(())
    }

    async fn init(&mut self, _socket: &mut TcpStream, _options: Arc<RunOptions>) {
        // do nothing by default, but can contain some preparation steps
    }

    fn get_processors() -> Vec<ProcessorFunction>;

    fn server_name<'a>() -> &'a str;

    fn port(options: Arc<RunOptions>) -> u16;
}
