mod handle_motion;

use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::processor::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

const MOVE_OPCODES: &[u32] = &[
    Opcode::MSG_MOVE_START_FORWARD as u32,
    Opcode::MSG_MOVE_START_BACKWARD as u32,
    Opcode::MSG_MOVE_JUMP as u32,
    Opcode::MSG_MOVE_HEARTBEAT as u32,
    Opcode::MSG_MOVE_START_TURN_LEFT as u32,
    Opcode::MSG_MOVE_START_TURN_RIGHT as u32,
    Opcode::MSG_MOVE_STOP as u32,
    Opcode::MSG_MOVE_STOP_STRAFE as u32,
    Opcode::MSG_MOVE_STOP_TURN as u32,
    Opcode::MSG_MOVE_START_PITCH_UP as u32,
    Opcode::MSG_MOVE_START_PITCH_DOWN as u32,
    Opcode::MSG_MOVE_STOP_PITCH as u32,
    Opcode::MSG_MOVE_FALL_LAND as u32,
    Opcode::MSG_MOVE_SET_PITCH as u32,
    Opcode::MSG_MOVE_START_SWIM as u32,
    Opcode::MSG_MOVE_STOP_SWIM as u32,
    Opcode::MSG_MOVE_SET_FACING as u32,
];

pub struct MovementProcessor;
impl Processor for MovementProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {

        let handlers: ProcessorResult = if MOVE_OPCODES.contains(&input.opcode) {
            vec![Box::new(handle_motion::Handler)]
        } else {
            vec![]
        };

        handlers
    }
}