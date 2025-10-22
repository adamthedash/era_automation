use super::components::*;
use std::f32;

use bevy::prelude::*;

use crate::{
    consts::GROUND_ITEM_BOB_HEIGHT,
    map::WorldPos,
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
