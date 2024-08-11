use std::collections::BTreeMap;
use async_trait::async_trait;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use tentacli_packet::{Segment, WorldPacket};
use tentacli_traits::types::custom_fields::PackedGuid;
use tentacli_traits::types::movement::{Movement, MovementExtraFlags, MovementFlags, MovementInfo, ObjectUpdateFlags, PositionInfo, UnitMoveType};
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::position::{Point3D, Vector3D};
use tentacli_traits::types::update_data::{BlockType, ObjectTypeID, UpdateData};
use tentacli_traits::types::update_fields::{FieldValue, ObjectField, PlayerField, UnitField};
use crate::primary::server::mock_data::CurrentPlayer;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

// #[derive(WorldPacket, Serialize, Deserialize, Debug)]
// struct Outcome {
//     data: Vec<u8>,
// }

#[derive(WorldPacket, Serialize, Debug)]
pub struct UpdateDataIncoming {
    pub blocks_amount: u32,
    #[depends_on(blocks_amount)]
    pub blocks: Vec<Block>,
}

#[derive(Segment, Debug, Clone, Default)]
pub struct Block {
    pub block_type: BlockType,
    #[conditional]
    pub guid: PackedGuid,
    #[conditional]
    pub object_type_id: ObjectTypeID,
    #[conditional]
    pub movement: Movement,
    #[conditional]
    pub update_data: UpdateData,
    #[conditional]
    pub guid_count: u32,
    #[depends_on(guid_count)]
    #[conditional]
    pub guids: Vec<PackedGuid>
}

impl Block {
    fn guid(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::VALUES |
            BlockType::MOVEMENT |
            BlockType::CREATE_OBJECT |
            BlockType::CREATE_OBJECT2
        )
    }

    fn object_type_id(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::CREATE_OBJECT |
            BlockType::CREATE_OBJECT2
        )
    }

    fn movement(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::MOVEMENT |
            BlockType::CREATE_OBJECT |
            BlockType::CREATE_OBJECT2
        )
    }

    fn update_data(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::VALUES |
            BlockType::CREATE_OBJECT |
            BlockType::CREATE_OBJECT2
        )
    }

    fn guid_count(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::NEAR_OBJECTS |
            BlockType::OUT_OF_RANGE_OBJECTS
        )
    }

    fn guids(instance: &mut Self) -> bool {
        matches!(
            instance.block_type.0,
            BlockType::NEAR_OBJECTS |
            BlockType::OUT_OF_RANGE_OBJECTS
        )
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut fields_amount = 1;

        if self.guid.0 != 0  {
            fields_amount += 1;
        }

        if !ObjectTypeID::is_none(&self.object_type_id) {
            fields_amount += 1;
        }

        if !Movement::is_empty(&self.movement) {
            fields_amount += 1;
        }

        if !UpdateData::is_empty(&self.update_data) {
            fields_amount += 1;
        }

        if self.guid_count > 0 {
            fields_amount += 2;
        }

        let mut state = serializer.serialize_struct("Block", fields_amount)?;
        state.serialize_field("block_type", &self.block_type)?;

        if self.guid.0 != 0  {
            state.serialize_field("guid", &self.guid)?;
        }

        if !ObjectTypeID::is_none(&self.object_type_id) {
            state.serialize_field("object_type_id", &self.object_type_id)?;
        }

        if !Movement::is_empty(&self.movement) {
            state.serialize_field("movement", &self.movement)?;
        }

        if !UpdateData::is_empty(&self.update_data) {
            state.serialize_field("update_data", &self.update_data)?;
        }

        if self.guid_count > 0 {
            state.serialize_field("guid_count", &self.guid_count)?;
            state.serialize_field("guids", &self.guids)?;
        }

        state.end()
    }
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();

        let block = Block {
            block_type: BlockType::new(BlockType::CREATE_OBJECT),
            guid: PackedGuid(CurrentPlayer::GUID),
            object_type_id: ObjectTypeID::new(ObjectTypeID::PLAYER),
            movement: {
                let mut movement = Movement::default();
                let movement_info = MovementInfo {
                    movement_flags: MovementFlags::NONE,
                    movement_extra_flags: MovementExtraFlags::NONE,
                    time: 0,
                    location: Vector3D {
                        point: Point3D {
                            x: CurrentPlayer::POS_X,
                            y: CurrentPlayer::POS_Y,
                            z: CurrentPlayer::POS_Z,
                        },
                        direction: 0.0,
                    },
                    ..MovementInfo::default()
                };

                movement.set_movement_info(movement_info);
                movement.movement_speed = {
                    let mut movement_speed: BTreeMap<u8, f32> = BTreeMap::new();

                    movement_speed.insert(UnitMoveType::MOVE_WALK, 2.5);
                    movement_speed.insert(UnitMoveType::MOVE_RUN, 7.0);
                    movement_speed.insert(UnitMoveType::MOVE_RUN_BACK, 4.5);
                    movement_speed.insert(UnitMoveType::MOVE_SWIM, 4.722222);
                    movement_speed.insert(UnitMoveType::MOVE_SWIM_BACK, 2.5);
                    movement_speed.insert(UnitMoveType::MOVE_TURN_RATE, std::f32::consts::PI);
                    movement_speed.insert(UnitMoveType::MOVE_FLIGHT, 7.0);
                    movement_speed.insert(UnitMoveType::MOVE_FLIGHT_BACK, 4.5);
                    movement_speed.insert(UnitMoveType::MOVE_TURN_RATE, std::f32::consts::PI);
                    movement_speed.insert(UnitMoveType::MOVE_PITCH_RATE, std::f32::consts::PI);

                    Some(movement_speed)
                };

                // this flag is necessary, otherwise origin client will wait for packet with SELF flag
                movement.object_update_flags.set(ObjectUpdateFlags::SELF, true);

                movement
            },
            update_data: UpdateData {
                object_fields: {
                    let mut map: BTreeMap<ObjectField, FieldValue> = BTreeMap::new();
                    map.insert(ObjectField::Guid, FieldValue::Long(CurrentPlayer::GUID));
                    map.insert(ObjectField::ScaleX, FieldValue::Float(1.0));
                    map.insert(ObjectField::Type, FieldValue::Integer(25));

                    map
                },
                unit_fields: {
                    let mut map: BTreeMap<UnitField, FieldValue> = BTreeMap::new();
                    map.insert(UnitField::AttackPower, FieldValue::Integer(26));
                    map.insert(UnitField::AuraState, FieldValue::Integer(4194304));
                    map.insert(UnitField::BaseAttackTime, FieldValue::IntegerArray(vec![1600, 1600]));
                    map.insert(UnitField::BaseHealth, FieldValue::Integer(25));
                    map.insert(UnitField::BaseMana, FieldValue::Integer(73));
                    map.insert(UnitField::BoundingRadius, FieldValue::Float(0.306));
                    map.insert(UnitField::Bytes0, FieldValue::Bytes(50332673));
                    map.insert(UnitField::Bytes1, FieldValue::Bytes(1));
                    map.insert(UnitField::CombatReach, FieldValue::Float(1.5));
                    map.insert(UnitField::DisplayId, FieldValue::Integer(CurrentPlayer::DISPLAY_ID));
                    map.insert(UnitField::FactionTemplate, FieldValue::Integer(1610));
                    map.insert(UnitField::Flags, FieldValue::Integer(262152));
                    map.insert(UnitField::Flags2, FieldValue::Integer(2048));
                    map.insert(UnitField::Health, FieldValue::Integer(55));
                    map.insert(UnitField::HoverHeight, FieldValue::Float(1.0));
                    map.insert(UnitField::Level, FieldValue::Integer(1));
                    map.insert(UnitField::MaxDamage, FieldValue::Float(4.971429));
                    map.insert(UnitField::MaxHealth, FieldValue::Integer(55));
                    map.insert(UnitField::MaxOffhandDamage, FieldValue::Float(2.4857144));
                    map.insert(UnitField::MaxPowers, FieldValue::IntegerArray(vec![0, 1000, 0, 100, 0, 0, 0]));
                    map.insert(UnitField::MaxRangedDamage, FieldValue::Float(4.6));
                    map.insert(UnitField::MinDamage, FieldValue::Float(3.97));
                    map.insert(UnitField::MinOffhandDamage, FieldValue::Float(1.98));
                    map.insert(UnitField::MinOffhandDamage, FieldValue::Float(1.98));
                    map.insert(UnitField::MinRangedDamage, FieldValue::Float(2.6));
                    map.insert(UnitField::ModCastSpeed, FieldValue::Float(1.0));
                    map.insert(UnitField::NativeDisplayId, FieldValue::Integer(CurrentPlayer::DISPLAY_ID));
                    map.insert(UnitField::Powers, FieldValue::IntegerArray(vec![0,0,0,100,0,0,0]));
                    map.insert(UnitField::RangedAttackPower, FieldValue::Integer(14));
                    map.insert(UnitField::RangedAttackTime, FieldValue::Integer(1600));
                    map.insert(UnitField::Resistances, FieldValue::IntegerArray(vec![48, 0, 0, 0, 0, 0, 0]));
                    map.insert(UnitField::Stats, FieldValue::IntegerArray(vec![48, 30, 30, 40, 50]));

                    map
                },
                player_fields: {
                    let mut map: BTreeMap<PlayerField, FieldValue> = BTreeMap::new();
                    map.insert(PlayerField::Bytes, FieldValue::BytesArray(vec![0, 16777216, 0]));
                    map.insert(PlayerField::CharacterPoints, FieldValue::IntegerArray(vec![0, 65537]));
                    map.insert(PlayerField::InvSlot, FieldValue::LongArray(vec![0u64; 23]));
                    map.insert(PlayerField::ExploredZones, FieldValue::BytesArray(vec![0u32; 128]));

                    map
                }
            },
            ..Block::default()
        };

        let blocks = vec![block];

        let packet = UpdateDataIncoming {
            blocks_amount: blocks.len() as u32,
            blocks,
        }.to_binary_with_server_opcode(Opcode::SMSG_UPDATE_OBJECT).unwrap();
        response.push(HandlerOutput::Data(packet));


        println!("SEND Opcode::SMSG_UPDATE_OBJECT");

        Ok(response)
    }
}