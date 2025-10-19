use std::f32;

use bevy::prelude::*;

use crate::{
    consts::{GROUND_ITEM_BOB_HEIGHT, GROUND_ITEM_BOB_SPEED, Z_GROUND_ITEM},
    map::WorldPos,
    player::{Player, Targettable},
    sprites::{ResourceSprite, SpriteSheets},
};

pub struct GroundItemPlugin;
impl Plugin for GroundItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_items)
            .add_systems(Update, drop_item);
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
    player: Single<&WorldPos, With<Player>>,
    inputs: Res<ButtonInput<KeyCode>>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    if inputs.just_pressed(KeyCode::KeyE) {
        commands.spawn((
            GroundItem,
            **player,
            Targettable,
            // Animation
            AnimationCycleTime(GROUND_ITEM_BOB_SPEED),
            // Render
            player.as_transform(Z_GROUND_ITEM),
            Sprite {
                image: sprite_sheets.resources.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheets.resources.layout.clone(),
                    index: ResourceSprite::Bowl as usize,
                }),
                ..Default::default()
            },
        ));
    }
}
