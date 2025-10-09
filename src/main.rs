#![feature(variant_count)]
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
        .add_plugins(VillagePlugin)
        .add_plugins(MapPlugin)
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

// /// Sync resource tilemap with underlying data
// fn update_resource_sprites(
//     query: Query<
//         (&mut TilemapChunkTileData, &ResourceData),
//         (With<ResourceLayer>, Changed<ResourceData>),
//     >,
//     resources: Query<&TerrainSprite>,
// ) {
//     for (mut dst, src) in query {
//         for (i, row) in src.0.iter().enumerate() {
//             for (j, entity) in row.iter().enumerate() {
//                 dst[i * row.len() + j] = entity.map(|entity| {
//                     let sprite = resources.get(entity).expect("Resource entity not found");
//
//                     TileData::from_tileset_index(*sprite as u16)
//                 })
//             }
//         }
//     }
// }
//
// /// Spawn the village centre building
// fn spawn_village(
//     chunks: Query<&Children, With<ChunkPos>>,
//     mut machine_layers: Query<&mut TilemapChunkTileData, With<MachineryLayer>>,
//     chunk_lut: Res<Chunks>,
// ) {
//     // Find the chunk
//     let (chunk, offset) = chunk_lut.chunk_for_tile(IVec2::ZERO);
//     let children = chunks.get(chunk).expect("Chunk not generated");
//     // Find the machinery layer
//     let mut data = children
//         .iter()
//         .find(|entity| machine_layers.get_mut(*entity).is_ok())
//         .and_then(|entity| machine_layers.get_mut(entity).ok())
//         .expect("MachineryLayer doesn't exist");
//
//     // Place the village
//     data[(offset.y * CHUNK_SIZE.y + offset.x) as usize] =
//         Some(TileData::from_tileset_index(TerrainSprite::House as u16));
// }
//
// #[derive(Component)]
// struct Targetted;
// /// Targets a resource in range of the player
// fn target_resource(
//     player: Single<&Transform, With<Player>>,
//     chunk_lut: Res<Chunks>,
//     chunks: Query<&Children, With<ChunkPos>>,
//     mut res_layers: Query<&mut TilemapChunkTileData, With<ResourceLayer>>,
// ) {
//     // Get player's chunk
//     let player_xy = player.translation.truncate();
//     let (chunk, offset) = chunk_lut.chunk_for_tile(player_xy.as_ivec2());
//     let children = chunks.get(chunk).expect("Chunk not generated");
//
//     // Find the layer
//     let mut data = children
//         .iter()
//         .find(|entity| res_layers.get_mut(*entity).is_ok())
//         .and_then(|entity| res_layers.get_mut(entity).ok())
//         .expect("ResourceLayer doesn't exist");
//
//     // Get the closest resource
// }
