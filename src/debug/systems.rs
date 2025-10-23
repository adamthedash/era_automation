use std::f32::consts::FRAC_PI_2;

use crate::{
    map::{ChunkPos, GradientData, TilePos},
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

use super::components::*;
use bevy::{
    math::{USizeVec2, ops::atan2},
    prelude::*,
};

pub fn toggle_gradient_arrows(mut enabled: ResMut<GradientArrowsEnabled>) {
    enabled.0 ^= true;
    if enabled.0 {
        info!("Showing gradient arrows");
    } else {
        info!("Hiding gradient arrows");
    }
}

pub fn despawn_gradient_arrows(mut arrows: ResMut<GradientArrowLUT>, mut commands: Commands) {
    for (_, arrow) in arrows.0.drain() {
        commands.entity(arrow).despawn();
    }
}

pub fn spawn_gradient_arrows(
    chunks: Query<(&ChunkPos, &GradientData)>,
    mut arrows_lut: ResMut<GradientArrowLUT>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (chunk_pos, gradients) in chunks {
        // info!("Spawning gradient arrows for chunk: {:?}", chunk_pos.0);
        let chunk_pos = chunk_pos.as_tile_pos();
        for (yo, row) in gradients.0.iter().enumerate() {
            for (xo, grad) in row.iter().enumerate() {
                let tile_pos = TilePos(chunk_pos.0 + USizeVec2::new(xo, yo).as_ivec2());
                if !arrows_lut.0.contains_key(&tile_pos) {
                    let transform = tile_pos
                        .as_transform(10.)
                        .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2))
                        .with_scale(Vec2::splat(grad.length().sqrt()).extend(1.));

                    // Spawn new arrow
                    let arrow = commands.spawn((transform, GradientArrow)).id();
                    ResourceSprite::DebugArrow.spawn_sprite(
                        &mut commands,
                        &sprite_sheets,
                        Some(arrow),
                    );

                    arrows_lut.0.insert(tile_pos, arrow);
                }
            }
        }
    }
}

pub fn update_gradient_arrows(
    chunks: Query<(&ChunkPos, &GradientData), Changed<GradientData>>,
    arrows_lut: ResMut<GradientArrowLUT>,
    mut commands: Commands,
) {
    for (chunk_pos, gradients) in chunks {
        info!("Updating gradient arrows for chunk: {:?}", chunk_pos.0);
        let chunk_pos = chunk_pos.as_tile_pos();
        for (yo, row) in gradients.0.iter().enumerate() {
            for (xo, grad) in row.iter().enumerate() {
                let tile_pos = TilePos(chunk_pos.0 + USizeVec2::new(xo, yo).as_ivec2());

                let entity = arrows_lut
                    .0
                    .get(&tile_pos)
                    .expect("Arrow entity should already exist at this point");

                let transform = tile_pos
                    .as_transform(10.)
                    .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2))
                    .with_scale(Vec2::splat(grad.length().sqrt()).extend(1.));

                commands.entity(*entity).insert(transform);
            }
        }
    }
}
