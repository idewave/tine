use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;
use futures::future::join_all;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::{HeaderDecryptor, HeaderEncryptor};
use crate::primary::crypto::srp::Srp;
use crate::primary::debug;
use crate::primary::server::types::Packet;
use crate::primary::types::{
    HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult,
};

#[derive(Default)]
pub struct RunOptions {
    pub srp: Arc<Mutex<Srp>>,
    pub port: u16,
    pub world_port: u16,
}

#[async_trait]
pub trait BaseServer: Send {
    async fn start(&mut self, options: RunOptions) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", options.port)).await?;

        let mut connections = vec![];

        loop {
            tokio::select! {
                Ok((socket, _)) = listener.accept() => {
                    let peer_addr = socket.peer_addr().unwrap();
                    {
                        let message = format!("Client connected: {:?}", peer_addr);
                        debug!("{}", message.yellow());
                    }

                    let (input_sender, input_receiver) = mpsc::channel::<Packet>(100);
                    let (output_sender, output_receiver) = mpsc::channel::<Vec<u8>>(100);
                    let (rx, tx) = socket.into_split();

                    let reader = BufReader::new(rx);
                    let writer = Arc::new(Mutex::new(tx));

                    connections.push(writer.clone());

                    let encryptor: Arc<Mutex<Option<HeaderEncryptor>>> = Arc::new(Mutex::new(None));
                    let decryptor: Arc<Mutex<Option<HeaderDecryptor>>> = Arc::new(Mutex::new(None));

                    let tasks = vec![
                        Self::handle_setup(
                            writer.clone()
                        ),
                        Self::handle_read(
                            input_sender,
                            reader,
                            decryptor.clone()
                        ),
                        Self::handle_input(
                            input_receiver,
                            output_sender,
                            options.srp.clone(),
                            encryptor.clone(),
                            decryptor.clone(),
                            options.world_port,
                        ),
                        Self::handle_write(
                            output_receiver,
                            writer.clone(),
                            encryptor.clone()
                        ),
                    ];

                    tokio::spawn(async move {
                        join_all(tasks).await;
                    });
                },
                _ = signal::ctrl_c() => {
                    for writer in connections.into_iter() {
                        writer.lock().await.shutdown().await?;
                    }

                    break;
                },
            }
        }

        Ok(())
    }

    fn handle_setup(writer: Arc<Mutex<OwnedWriteHalf>>) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            Self::setup(writer.clone()).await
        })
    }

    fn handle_read(
        input_sender: Sender<Packet>,
        reader: BufReader<OwnedReadHalf>,
        decryptor: Arc<Mutex<Option<HeaderDecryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>>;

    fn handle_input(
        mut input_receiver: Receiver<Packet>,
        output_sender: Sender<Vec<u8>>,
        srp: Arc<Mutex<Srp>>,
        encryptor: Arc<Mutex<Option<HeaderEncryptor>>>,
        decryptor: Arc<Mutex<Option<HeaderDecryptor>>>,
        world_port: u16,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            while let Some(packet) = input_receiver.recv().await {
                let Packet { body: data, opcode } = packet;
                debug!(
                    "[RECV]: {}",
                    Opcode::get_opcode_name(opcode).unwrap().blue()
                );

                let mut input = HandlerInput {
                    data,
                    opcode,
                    srp: Arc::clone(&srp),
                    world_port,
                };

                let handler_list = Self::get_processors()
                    .iter()
                    .flat_map(|processor| processor(&mut input))
                    .collect::<ProcessorResult>();

                for mut handler in handler_list {
                    let outputs = handler.handle(&mut input).await?;
                    for output in outputs {
                        match output {
                            HandlerOutput::Data(opcode, packet) => {
                                debug!(
                                    "[SEND]: {}",
                                    Opcode::get_opcode_name(opcode as u32).unwrap().magenta()
                                );
                                output_sender.send(packet).await?;
                            },
                            HandlerOutput::HeaderCrypt(enc, dec) => {
                                *encryptor.lock().await = Some(enc);
                                *decryptor.lock().await = Some(dec);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn handle_write(
        mut output_receiver: Receiver<Vec<u8>>,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        encryptor: Arc<Mutex<Option<HeaderEncryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            loop {
                if let Some(mut packet) = output_receiver.recv().await {
                    if let Some(header_crypt) = encryptor.lock().await.as_mut() {
                        let is_large_packet = packet[0] >= 0x80;
                        let header_size: usize = if is_large_packet { 5 } else { 4 };

                        let encrypted_header = header_crypt.encrypt(&packet[..header_size]);
                        packet[..header_size].copy_from_slice(&encrypted_header);
                    }

                    writer.lock().await.write_all(&packet).await?;
                }
            }
        })
    }

    async fn setup(_writer: Arc<Mutex<OwnedWriteHalf>>) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_processors() -> Vec<ProcessorFunction>;

    fn server_name<'a>() -> &'a str;
}