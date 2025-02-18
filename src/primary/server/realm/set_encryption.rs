use async_trait::async_trait;

use crate::primary::crypto::header_crypt::{HeaderDecryptor, HeaderEncryptor};
use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let session_key = {
            let guard = input.srp.lock().await;
            guard.session_key.as_ref().unwrap().clone()
        };

        Ok(vec![
            HandlerOutput::HeaderCrypt(
                HeaderEncryptor::new(&session_key),
                HeaderDecryptor::new(&session_key),
            )
        ])
    }
}
