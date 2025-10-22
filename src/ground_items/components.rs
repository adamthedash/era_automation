use std::f32;

use bevy::prelude::*;

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
