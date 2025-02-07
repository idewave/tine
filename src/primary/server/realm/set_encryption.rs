use async_trait::async_trait;

use crate::primary::crypto::header_crypt::HeaderCrypt;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerResult};

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
