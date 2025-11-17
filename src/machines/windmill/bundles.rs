use bevy::prelude::*;

use super::super::components::*;
use crate::{consts::Z_RESOURCES, map::TilePos, player::Targettable, sprites::EntitySprite};

/// For windmill machines at all times (stateless parts of the prefab)
#[derive(Bundle)]
pub struct WindmillBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    windmill_marker: Windmill,
    // Speed is just used for animation for windmills
    speed: MachineSpeed,
}
impl WindmillBundle {
    pub fn new(speed: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Windmill,
            animation_sprites: AnimationSprites(sprites),
            windmill_marker: Windmill,
            speed: MachineSpeed(speed),
        }
    }
}

/// For windmill machines when placed down (stateful / instance-specific components)
#[derive(Bundle)]
pub struct PlacedWindmillBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
    current_production: PowerProduction,
    // State is just used for animation for windmills
    state: MachineState,
}
impl PlacedWindmillBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            placed: Placed,
            targettable: Targettable,
            current_production: PowerProduction(0.0),
            state: MachineState(0.),
        }
    }
}
