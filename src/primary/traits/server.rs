use std::sync::{Arc, Mutex as SyncMutex};
use std::time::Duration;
use anyhow::{Result as AnyResult};
use async_trait::async_trait;
use colored::Colorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::primary::crypto::srp::Srp;
use crate::primary::types::{HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult};

pub struct RunOptions {
    pub srp: Arc<SyncMutex<Srp>>,
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
                        Ok((socket, _)) => {
                            let peer_addr = socket.peer_addr().unwrap();
                            {
                                let message = format!("Client connected: {:?}", peer_addr);
                                println!("{}", message.yellow());
                            }

                            let shutdown_tx = shutdown_tx.clone();
                            let options = options.clone();
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

    async fn handle_connection(mut socket: TcpStream, options: Arc<RunOptions>) -> AnyResult<()> {
        Self::init(&mut socket).await;

        loop {
            let mut buf = [0; 65536];
            match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("{}", "Client disconnected".yellow());
                    break;
                }
                Ok(n) => {
                    {
                        let message = format!("Received {} bytes: {:?}", n, &buf[..n]);
                        println!("{}", message.yellow());
                    }
                    let packet = &buf[..n];

                    let mut input = Self::generate_input(packet, &options);

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
                                        HandlerOutput::Data(packet) => {
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
                }
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                    break;
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn init(_socket: &mut TcpStream) {
        // do nothing by default, but can contain some preparation steps
    }

    fn generate_input(packet: &[u8], options: &RunOptions) -> HandlerInput;

    fn get_processors() -> Vec<ProcessorFunction>;

    fn host<'a>() -> &'a str;

    fn port() -> u16;

    fn server_name<'a>() -> &'a str;
}