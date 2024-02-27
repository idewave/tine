use std::collections::BTreeMap;
use std::sync::{Arc, Mutex as SyncMutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use colored::*;
use crate::primary::crypto::srp::Srp;

mod opcodes;
mod auth;

use crate::primary::server::auth::AuthProcessor;
use crate::primary::shared::session::Session;
use crate::primary::traits::processor::Processor;
use crate::primary::types::{HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 3724;

type SessionKey = Vec<u8>;
type Sessions = BTreeMap<String, Option<SessionKey>>;

pub struct LoginServer {
    _sessions: Arc<Mutex<Sessions>>,
}

impl LoginServer {
    pub fn new() -> Self {
        Self {
            _sessions: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub async fn run(&mut self) {
        let listener = TcpListener::bind(format!("{}:{}", HOST, PORT)).await.unwrap();
        println!("LoginServer is started on port {}", PORT);

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let sessions = Arc::clone(&self._sessions);

            tokio::spawn(async move {
                let peer_addr = socket.peer_addr().unwrap();
                println!("Client connected: {:?}", peer_addr);

                let cloned_sessions = sessions.clone();
                cloned_sessions.lock().await.entry(peer_addr.to_string()).or_insert(None);
                let srp = Arc::new(SyncMutex::new(Srp::new()));

                loop {
                    let mut buf = [0; 65536];
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            println!("Received {} bytes: {:?}", n, &buf[..n]);
                            let packet = &buf[..n];

                            let mut input = HandlerInput {
                                data: packet[1..].to_vec(),
                                opcode: packet[0] as u16,
                                srp: Arc::clone(&srp),
                            };

                            let processors = Self::get_login_processors();
                            let handler_list = processors
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
                                                HandlerOutput::SessionKey(key) => {},
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
                }
            });
        }
    }

    fn get_login_processors() -> Vec<ProcessorFunction> {
        vec![
            Box::new(AuthProcessor::get_handlers),
        ]
    }
}

pub struct WorldServer {}
impl WorldServer {
    pub async fn run() {
        let listener = TcpListener::bind(format!("{}:{}", HOST, PORT)).await.unwrap();
        println!("LoginServer is started on port {}", PORT);

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                println!("Client connected: {:?}", socket.peer_addr().unwrap());

                loop {
                    let mut buf = [0; 1024];
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            println!("Received {} bytes: {:?}", n, &buf[..n]);
                            let mut packet = vec![n as u8];
                            packet.extend_from_slice(&buf[..n]);

                            if let Err(e) = socket.write_all(&packet).await {
                                eprintln!("Error writing to socket: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }
}