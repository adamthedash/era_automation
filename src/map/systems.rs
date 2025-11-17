use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use super::components::*;
use crate::{consts::CHUNK_SIZE, ground_items::GroundItem, utils::noise::perlin_stack};

/// Set up the world generation
pub fn init_world_gen(mut commands: Commands) {
    commands.insert_resource(WorldGenerator {
        height: Box::new(perlin_stack(43, 4, 1., 0.5, 1. / 64., 0.)),
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
                    - Vec2::new(1.5, 0.5))
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
        chunk.trigger(|entity| ChunkCreated {
            entity,
            pos: *chunk_pos,
        });
    }
}

/// Update the transforms when WorldPos changes
pub fn update_transforms(
    query: Query<(&WorldPos, &mut Transform), (Changed<WorldPos>, Without<GroundItem>)>,
) {
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
                // Transforms are Up-positive, tilemaps are down-positive, so need to flip the data
                // when rendering

                let index = (CHUNK_SIZE.y as usize - i - 1) * CHUNK_SIZE.x as usize + j;
                tilemap_data[index] = Some(TileData::from_tileset_index(terrain as u16));
            }
        }
    }
}

/// Updates the gradient map for adjacent chunks
pub fn update_gradient_map(
    event: On<ChunkCreated>,
    chunk_lut: Res<ChunkLUT>,
    mut writer: MessageWriter<RecomputeGradient>,
) {
    // Trigger gradient recalc for this and adjacent chunks
    for i in -1..=1 {
        for j in -1..=1 {
            let chunk_pos = event.pos + IVec2::new(i, j);

            if chunk_lut.0.contains_key(&chunk_pos) {
                writer.write(RecomputeGradient(chunk_pos));
            }
        }
    }
}

/// For doing convolutions across chunk boundaries
struct PaddedHeightGrid<'a>([[Option<&'a HeightData>; 3]; 3]);
impl PaddedHeightGrid<'_> {
    /// Get a value relative to the central chunk. If an adjacent chunk is not available, 0 is
    /// returned instead
    fn get(&self, i: i32, j: i32) -> f32 {
        let chunk_i = i.div_euclid(CHUNK_SIZE.y as i32) + 1;
        let offset_i = i.rem_euclid(CHUNK_SIZE.y as i32);

        let chunk_j = j.div_euclid(CHUNK_SIZE.x as i32) + 1;
        let offset_j = j.rem_euclid(CHUNK_SIZE.x as i32);

        if let Some(data) = self.0[chunk_i as usize][chunk_j as usize] {
            data.0[offset_i as usize][offset_j as usize]
        } else {
            0.
        }
    }
}

/// Recomputes the height gradients for a chunk
pub fn recompute_gradients(
    mut messages: MessageReader<RecomputeGradient>,
    chunk_data: Chunks<(&ChunkPos, &HeightData)>,
    mut grad_data: Chunks<&mut GradientData>,
) {
    for RecomputeGradient(chunk_pos) in messages.read() {
        info!("Recomputing gradients for chunk: {:?}", chunk_pos.0);

        // Build padded height map
        let mut padded_height_map = PaddedHeightGrid([[None; 3]; 3]);
        for i in 0..3 {
            for j in 0..3 {
                let chunk_pos = chunk_pos + IVec2::new(j - 1, i - 1);
                if let Some((_, height_data)) = chunk_data.get(&chunk_pos) {
                    padded_height_map.0[i as usize][j as usize] = Some(height_data);
                }
            }
        }

        // Get gradient data to write to for this chunk
        let mut gradient_data = grad_data
            .get_mut(chunk_pos)
            .expect("Chunk has just been created, so these should exist");

        // Compute gradients - sobel filter
        let h = padded_height_map;
        for i in 0..CHUNK_SIZE.y as i32 {
            for j in 0..CHUNK_SIZE.x as i32 {
                let gy1 = h.get(i + 1, j - 1) + 2. * h.get(i + 1, j) + h.get(i + 1, j + 1);
                let gy0 = h.get(i - 1, j - 1) + 2. * h.get(i - 1, j) + h.get(i - 1, j + 1);
                let gy = gy1 - gy0;

                let gx1 = h.get(i - 1, j + 1) + 2. * h.get(i, j + 1) + h.get(i + 1, j + 1);
                let gx0 = h.get(i - 1, j - 1) + 2. * h.get(i, j - 1) + h.get(i + 1, j - 1);
                let gx = gx1 - gx0;

                gradient_data.0[i as usize][j as usize] = Vec2::new(gx, gy);
            }
        }
    }
}
