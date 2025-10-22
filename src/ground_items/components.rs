use std::f32;

use bevy::prelude::*;

use crate::{
    consts::{GROUND_ITEM_BOB_SPEED, Z_GROUND_ITEM},
    map::WorldPos,
    player::Targettable,
};

/// Marker trait for items on the ground
#[derive(Component)]
pub struct GroundItem;

/// The duration along the animation's total time in seconds
#[derive(Component, Default)]
pub struct AnimationTime(pub f32);

/// Total time for animation loop to play in seconds
#[derive(Component)]
#[require(AnimationTime)]
pub struct AnimationCycleTime(pub f32);

#[derive(Bundle)]
pub struct GroundItemBundle {
    ground_marker: GroundItem,
    target_marker: Targettable,
    world_pos: WorldPos,
    animation: AnimationCycleTime,
    transform: Transform,
}
impl GroundItemBundle {
    pub fn new(world_pos: &WorldPos) -> Self {
        Self {
            ground_marker: GroundItem,
            target_marker: Targettable,
            world_pos: *world_pos,
            animation: AnimationCycleTime(GROUND_ITEM_BOB_SPEED),
            transform: world_pos.as_transform(Z_GROUND_ITEM),
        }
    }
}
