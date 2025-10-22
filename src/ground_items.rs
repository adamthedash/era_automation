use std::f32;

use bevy::prelude::*;

use crate::{
    consts::{GROUND_ITEM_BOB_HEIGHT, GROUND_ITEM_BOB_SPEED, Z_GROUND_ITEM},
    map::WorldPos,
    player::{HeldBy, Holding, Player, Targettable, Targetted, held_item_bundle},
    utils::run_if::{empty_hands, key_just_pressed},
};

pub struct GroundItemPlugin;
impl Plugin for GroundItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_items,
                (drop_item, pickup_item.run_if(empty_hands))
                    .run_if(key_just_pressed(KeyCode::KeyE)),
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
        let offset_transform =
            WorldPos(world_pos.0 + Vec2::Y * offset).as_transform(transform.translation.z);
        transform.translation = offset_transform.translation;
    }
}

/// Drop an item on the ground
fn drop_item(player: Single<(&WorldPos, &Holding), With<Player>>, mut commands: Commands) {
    info!("Dropping item");

    let held_item = player
        .1
        .iter()
        .next()
        .expect("This system only runs if there is an item being held");

    // Move entity from player children to world
    let mut item = commands.entity(held_item);
    item.remove::<ChildOf>();

    // Remove holding related components
    item.remove::<HeldBy>();

    // Add ground related components
    item.insert((
        // Markers
        GroundItem,
        Targettable,
        // Drop at player's feet
        *player.0,
        // Animation
        AnimationCycleTime(GROUND_ITEM_BOB_SPEED),
        // Render
        player.0.as_transform(Z_GROUND_ITEM),
    ));
}

/// Pick up a nearby item from the ground
fn pickup_item(
    ground_item: Single<Entity, (With<GroundItem>, With<Targetted>)>,
    player: Single<(Entity, &Transform), With<Player>>,
    mut commands: Commands,
) {
    info!("Picking up item");
    commands
        .entity(*ground_item)
        // Remove ground related components
        .remove::<(
            GroundItem,
            Targettable,
            // TODO: See if this can be removed just by removing Targettable, rather than having it
            // be automatically cleaned up on next frame by targetting system
            Targetted,
            AnimationCycleTime,
            WorldPos,
        )>()
        // Add holding related components
        .insert(held_item_bundle(player.0));
}
