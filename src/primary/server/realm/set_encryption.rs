use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;

use crate::primary::crypto::header_crypt::HeaderCrypt;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerResult};

// Opcode::CMSG_AUTH_SESSION
#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Income {
    build: u32,
    unknown: u32,
    account: String,
    unknown2: u32,
    client_seed: [u8; 4],
    unknown3: u64,
    server_id: u32,
    unknown4: u64,
    digest: [u8; 20],
    // addons_count: u32,
    // addons: Vec<u8>,
}

// Opcode::SMSG_AUTH_RESPONSE
#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    status: u8,
    billing_time_remaining: u32,
    billing_plan_flags: u8,
    billing_time_rested: u32,
    expansion: u8,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let response = Vec::new();
        let session_key = {
            let guard = input.srp.lock().await;
            guard.session_key.as_ref().unwrap().clone()
        };

        let mut guard = input.connection.lock().await;
        guard.header_crypt = Some(HeaderCrypt::new(&session_key));

        Ok(response)
    }
}
