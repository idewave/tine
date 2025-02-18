use async_trait::async_trait;

use crate::primary::types::{HandlerInput, HandlerResult, ProcessorResult};

pub mod server;

#[async_trait]
pub trait PacketHandler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult;
}

pub trait Processor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult;
}
