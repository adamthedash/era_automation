mod components;
pub mod systems;

use std::f32::consts::FRAC_PI_2;

use bevy::{
    math::{USizeVec2, ops::atan2},
    platform::collections::HashMap,
    prelude::*,
};

pub use components::*;
use systems::*;

use crate::sprites::{GetSprite, ResourceSprite, SpriteSheets};

/// Controls world generation
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_world_gen)
            .add_systems(
                Update,
                (
                    create_chunks,
                    update_transforms,
                    update_tilemap_data,
                    spawn_gradient_arrows,
                ),
            )
            .add_systems(FixedUpdate, recompute_gradients)
            .add_observer(update_gradient_map)
            .init_resource::<ChunkLUT>()
            .init_resource::<GradientArrowLUT>()
            .add_message::<CreateChunk>()
            .add_message::<RecomputeGradient>();
    }
}

#[derive(Component)]
struct GradientArrow;

#[derive(Resource, Default)]
struct GradientArrowLUT(HashMap<TilePos, Entity>);

/// Show gradient of hills with arrow overlays
/// TODO: move to debug plugin
fn spawn_gradient_arrows(
    chunks: Query<(&ChunkPos, &GradientData), Changed<GradientData>>,
    mut arrows_lut: ResMut<GradientArrowLUT>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (chunk_pos, gradients) in chunks {
        info!("Updating gradient arrows for chunk: {:?}", chunk_pos.0);
        let chunk_pos = chunk_pos.as_tile_pos();
        for (yo, row) in gradients.0.iter().enumerate() {
            for (xo, grad) in row.iter().enumerate() {
                let tile_pos = TilePos(chunk_pos.0 + USizeVec2::new(xo, yo).as_ivec2());
                let transform = tile_pos
                    .as_transform(10.)
                    .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2))
                    .with_scale(Vec2::splat(grad.length().sqrt()).extend(1.));

                if let Some(entity) = arrows_lut.0.get(&tile_pos) {
                    // Already exists, update transform instead
                    commands.entity(*entity).insert(transform);
                } else {
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
