#![feature(variant_count)]
#![allow(clippy::type_complexity)]
mod consts;
mod map;
mod player;
mod resources;
mod sprites;
mod utils;
mod village;

use bevy::prelude::*;

use crate::{
    consts::CHUNK_LOAD_RADIUS,
    map::{ChunkLUT, ChunkPos, CreateChunk, MapPlugin},
    player::{Player, PlayerPlugin},
    resources::ResourcePlugin,
    sprites::SpritePlugin,
    village::VillagePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpritePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(VillagePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ResourcePlugin)
        .add_systems(Update, (spawn_chunks))
        .run();
}

/// Spawn chunks around the player if they're not generated yet
fn spawn_chunks(
    player: Single<&Transform, With<Player>>,
    mut messages: MessageWriter<CreateChunk>,
    chunk_lut: Res<ChunkLUT>,
) {
    let player_chunk = ChunkPos::from_transform(&player);

    for x in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
        for y in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
            let chunk_pos = ChunkPos(IVec2::new(player_chunk.0.x + x, player_chunk.0.y + y));
            if !chunk_lut.0.contains_key(&chunk_pos) {
                messages.write(CreateChunk(chunk_pos));
            }
        }
    }
}
