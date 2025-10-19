use std::f32;

use bevy::prelude::*;

use crate::{
    consts::{GROUND_ITEM_BOB_HEIGHT, GROUND_ITEM_BOB_SPEED, Z_GROUND_ITEM},
    map::WorldPos,
    player::{HeldItem, Player, Targettable, Targetted},
    utils::run_if::key_just_pressed,
};

pub struct GroundItemPlugin;
impl Plugin for GroundItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_items,
                drop_item.run_if(key_just_pressed(KeyCode::KeyE)),
            ),
        );
    }
}

#[derive(Component)]
pub struct GroundItem;

/// The duration along the animation's total time
#[derive(Component, Default)]
pub struct AnimationTime(f32);
#[derive(Component)]
#[require(AnimationTime)]
pub struct AnimationCycleTime(f32);

/// Bob the items up & down
fn animate_items(
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
        *transform = WorldPos(world_pos.0 + Vec2::Y * offset).as_transform(Z_GROUND_ITEM);
    }
}

/// Drop an item on the ground
fn drop_item(
    player_pos: Single<&WorldPos, With<Player>>,
    held_item: Single<Entity, With<HeldItem>>,
    mut commands: Commands,
) {
    // Move entity from player children to world
    let mut item = commands.entity(*held_item);
    item.remove::<ChildOf>();

    // Remove holding related components
    // TODO: Move this to a bundle
    item.remove::<HeldItem>();

    // Add ground related components
    item.insert((
        // Markers
        GroundItem,
        Targettable,
        // Drop at player's feet
        **player_pos,
        // Animation
        AnimationCycleTime(GROUND_ITEM_BOB_SPEED),
        // Render
        player_pos.as_transform(Z_GROUND_ITEM),
    ));
}

/// Pick up a nearby item from the ground
fn pickup_item(
    ground_item: Single<(), (With<GroundItem>, With<Targetted>)>,
    held_item: Query<(), With<HeldItem>>,
) {
    if !held_item.is_empty() {
        // Already holding an item
    }
}
