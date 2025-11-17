use bevy::prelude::*;

use super::super::components::*;
use crate::{consts::Z_RESOURCES, map::TilePos, player::Targettable, sprites::EntitySprite};

/// For picker-upper machines at all times
#[derive(Bundle)]
pub struct PickerUpperBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    pickerupper_marker: PickerUpper,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
}
impl PickerUpperBundle {
    pub fn new(speed: f32, power_consumption: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::PickerUpper,
            animation_sprites: AnimationSprites(sprites),
            pickerupper_marker: PickerUpper,
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
        }
    }
}

/// For picker-upper machines when placed down
#[derive(Bundle)]
pub struct PlacedPickerUpperBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
    pickup_state: MachineState,
}
impl PlacedPickerUpperBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            placed: Placed,
            targettable: Targettable,
            pickup_state: MachineState(0.),
        }
    }
}
