use super::components::*;
use std::f32;

use bevy::prelude::*;

use crate::{
    consts::{GROUND_ITEM_BOB_HEIGHT, ITEM_ROLL_SPEED},
    map::{ChunkLUT, GradientData, WorldPos},
    player::{HeldItemBundle, Holding, Player, Targetted},
};

/// Bob the items up & down
pub fn animate_items(
    mut items: Query<
        (
            &WorldPos,
            &mut Transform,
            &mut AnimationTime,
            &AnimationCycleTime,
        ),
        With<GroundItem>,
    >,
    timer: Res<Time>,
) {
    for (world_pos, mut transform, mut current_time, total_time) in &mut items {
        // Advance animation
        current_time.0 = (current_time.0 + timer.delta_secs()) % total_time.0;

        // Calculate bob offset
        let offset = (current_time.0 / total_time.0 * f32::consts::PI * 2.).sin()
            * GROUND_ITEM_BOB_HEIGHT
            / 2.;

        // Update transform
        let offset_transform =
            WorldPos(world_pos.0 + Vec2::Y * offset).as_transform(transform.translation.z);
        transform.translation = offset_transform.translation;
    }
}

/// Drop an item on the ground
pub fn drop_item(player: Single<(&WorldPos, &Holding), With<Player>>, mut commands: Commands) {
    info!("Dropping item");

    let held_item = player
        .1
        .iter()
        .next()
        .expect("This system only runs if there is an item being held");

    commands
        .entity(held_item)
        // Remove holding related components
        .remove::<HeldItemBundle>()
        // Add ground related components
        .insert(GroundItemBundle::new(player.0));
}

/// Pick up a nearby item from the ground
pub fn pickup_item(
    ground_item: Single<Entity, (With<GroundItem>, With<Targetted>)>,
    player: Single<(Entity, &Transform), With<Player>>,
    mut commands: Commands,
) {
    info!("Picking up item");
    commands
        .entity(*ground_item)
        // Remove ground related components
        .remove::<GroundItemBundle>()
        // Add holding related components
        .insert(HeldItemBundle::new(player.0));
}

/// Roll items according to the terrain gradient
pub fn roll_items(
    items: Query<&mut WorldPos, With<GroundItem>>,
    chunk_lut: Res<ChunkLUT>,
    gradients: Query<&GradientData>,
    timer: Res<Time>,
) {
    for mut item_pos in items {
        let (chunk_pos, offset) = item_pos.tile().to_chunk_offset();

        let chunk = chunk_lut
            .0
            .get(&chunk_pos)
            .expect("Chunk should already be generated if there's an item on the ground");

        let gradients = gradients
            .get(*chunk)
            .expect("Gradient data should already be available if chunk is created");

        let tile_gradient = gradients.0[offset.y as usize][offset.x as usize];

        // Roll the item dowmhill
        // TODO: Friction, rollability, etc.
        item_pos.0 -= tile_gradient * timer.delta_secs() * ITEM_ROLL_SPEED;
    }
}
