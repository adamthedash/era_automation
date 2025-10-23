mod components;
pub mod systems;

use std::f32::consts::FRAC_PI_2;

use bevy::{
    math::{USizeVec2, ops::atan2},
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
                (create_chunks, update_transforms, update_tilemap_data),
            )
            .init_resource::<ChunkLUT>()
            .add_observer(spawn_gradient_arrows)
            .add_message::<CreateChunk>();
    }
}

/// Debug arrows
fn spawn_gradient_arrows(
    event: On<ChunkCreated>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    chunks: Query<(&ChunkPos, &GradientData)>,
) {
    let Ok((chunk_pos, gradients)) = chunks.get(event.0) else {
        return;
    };

    let chunk_pos = chunk_pos.as_tile_pos();
    for (yo, row) in gradients.0.iter().enumerate() {
        for (xo, grad) in row.iter().enumerate() {
            let transform = TilePos(chunk_pos.0 + USizeVec2::new(xo, yo).as_ivec2())
                .as_transform(10.)
                .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2));

            let arrow = commands.spawn((transform,)).id();

            ResourceSprite::DebugArrow.spawn_sprite(&mut commands, &sprite_sheets, Some(arrow));
        }
    }
}
