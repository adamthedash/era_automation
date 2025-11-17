use bevy::prelude::*;

use super::super::components::*;
use crate::{
    consts::{Z_RESOURCES, Z_TRANSPORTED_ITEM},
    map::TilePos,
    player::Targettable,
    sprites::EntitySprite,
};

/// For transporter machines at all times
#[derive(Bundle)]
pub struct TransporterBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    transporter_marker: Transporter,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
    accepts_items: AcceptsItems,
}
impl TransporterBundle {
    pub fn new(speed: f32, power_consumption: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Transporter,
            transporter_marker: Transporter,
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
            animation_sprites: AnimationSprites(sprites),
            accepts_items: AcceptsItems::Any,
        }
    }
}

/// For transporter machines when placed down
#[derive(Bundle)]
pub struct PlacedTransporterBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
}
impl PlacedTransporterBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            targettable: Targettable,
            placed: Placed,
        }
    }
}

/// For items on conveyor belts
#[derive(Bundle)]
pub struct TransportedItemBundle {
    parent: ChildOf,
    transporter: TransportedBy,
    transport_state: MachineState,
    transform: Transform,
}
impl TransportedItemBundle {
    pub fn new(transporter: Entity, transporter_direction: &Direction) -> Self {
        Self {
            parent: ChildOf(transporter),
            transporter: TransportedBy(transporter),
            transport_state: MachineState(0.),
            transform: Transform::from_translation(
                (transporter_direction.0.as_vec2() * 0.5).extend(Z_TRANSPORTED_ITEM),
            ),
        }
    }
}
