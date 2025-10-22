use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{consts::CHUNK_SIZE, utils::noise::perlin_stack};

use super::components::*;

/// Set up the world generation
pub fn init_world_gen(mut commands: Commands) {
    commands.insert_resource(WorldGenerator {
        terrain: Box::new(perlin_stack(42, 4, 1., 0.5, 1. / 16., 0.)),
    });
}
/// Initialises a new chunk at a given position
pub fn create_chunks(
    mut messages: MessageReader<CreateChunk>,
    mut commands: Commands,
    mut chunk_lut: ResMut<ChunkLUT>,
    asset_server: Res<AssetServer>,
    generator: Res<WorldGenerator>,
) {
    for CreateChunk(chunk_pos) in messages.read() {
        info!("Spawning chunk entity: {:?}", chunk_pos.0);
        let mut chunk = commands.spawn((
            *chunk_pos,
            Transform::from_translation(
                // +0.5 chunks so TileMapChunk is rendered from its origin
                // -0.5 tiles so resource sprites are aligned properly
                ((chunk_pos.0.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE.as_vec2()
                    - Vec2::splat(0.5))
                .extend(0.),
            ),
            Visibility::default(),
            // Terrain data
            generator.generate_terrain(*chunk_pos),
            // Render
            TilemapChunk {
                chunk_size: CHUNK_SIZE,
                // Render tiles to unit square
                tile_display_size: UVec2::ONE,
                tileset: asset_server.load("terrain_sheet.png"),
                ..Default::default()
            },
            TilemapChunkTileData(vec![None; CHUNK_SIZE.element_product() as usize]),
        ));

        chunk_lut.0.insert(*chunk_pos, chunk.id());
        chunk.trigger(ChunkCreated);
    }
}

/// Update the transforms when WorldPos changes
pub fn update_transforms(query: Query<(&WorldPos, &mut Transform), Changed<WorldPos>>) {
    for (world_pos, mut transform) in query {
        let new_transform = world_pos.as_transform(transform.translation.z);
        transform.translation = new_transform.translation;
    }
}

/// Sync terrain game data to the tilemap for rendering
pub fn update_tilemap_data(
    query: Query<(&TerrainData, &mut TilemapChunkTileData), Changed<TerrainData>>,
) {
    for (tile_data, mut tilemap_data) in query {
        for (i, row) in tile_data.0.iter().enumerate() {
            for (j, &terrain) in row.iter().enumerate() {
                //

                let index = (CHUNK_SIZE.y as usize - i - 1) * CHUNK_SIZE.x as usize + j;
                tilemap_data[index] = Some(TileData::from_tileset_index(terrain as u16));
            }
        }
    }
}
