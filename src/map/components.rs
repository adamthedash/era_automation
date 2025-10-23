use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    consts::{CHUNK_SIZE, TERRAIN_STARTING_RADIUS},
    sprites::TerrainSprite,
    utils::{
        math::{lerp_f32, lerp_f64},
        noise::MyGenerator,
    },
};

/// Discrete tile locations - World space
#[derive(Component, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TilePos(pub IVec2);
impl TilePos {
    /// Convert to transform in display space
    pub fn as_transform(&self, z: f32) -> Transform {
        self.as_world_pos().as_transform(z)
    }

    pub fn as_world_pos(&self) -> WorldPos {
        WorldPos(self.0.as_vec2())
    }

    pub fn to_chunk_offset(&self) -> (ChunkPos, UVec2) {
        let chunk = ChunkPos(self.0.div_euclid(CHUNK_SIZE.as_ivec2()));
        let offset = self.0.rem_euclid(CHUNK_SIZE.as_ivec2()).as_uvec2();

        (chunk, offset)
    }
}

/// Discrete chunk locations - Chunk space
#[derive(Component, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkPos(pub IVec2);
impl ChunkPos {
    pub fn as_tile_pos(&self) -> TilePos {
        TilePos(self.0 * CHUNK_SIZE.as_ivec2())
    }
}

/// Continuous locations - World space
#[derive(Component, Clone, Copy)]
pub struct WorldPos(pub Vec2);
impl WorldPos {
    pub fn as_transform(&self, z: f32) -> Transform {
        Transform::from_translation(self.0.extend(z))
    }

    pub fn chunk(&self) -> ChunkPos {
        ChunkPos(self.0.div_euclid(CHUNK_SIZE.as_vec2()).as_ivec2())
    }
}

/// Game data friendly storage for terrain tiles
#[derive(Component, Default)]
pub struct TerrainData(pub [[TerrainSprite; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize]);

/// Directional flow of hills
#[derive(Component, Default)]
pub struct HeightData(pub [[f32; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize]);

/// Directional flow of hills
#[derive(Component, Default)]
pub struct GradientData(pub [[Vec2; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize]);

/// Random generation for everything in the world
#[derive(Resource)]
pub struct WorldGenerator {
    // Height
    pub height: Box<dyn MyGenerator<2>>,
}
impl WorldGenerator {
    pub fn generate_terrain(&self, pos: ChunkPos) -> impl Bundle {
        let world_pos = pos.as_tile_pos();

        // Generate height map
        let mut height_data = HeightData::default();
        for yo in 0..CHUNK_SIZE.y {
            for xo in 0..CHUNK_SIZE.x {
                let pos = world_pos.0 + UVec2::new(xo, yo).as_ivec2();

                height_data.0[yo as usize][xo as usize] =
                    self.height.sample([pos.x as f64, pos.y as f64]) as f32;
            }
        }

        // Generate terrain map
        let mut terrain_data = TerrainData::default();
        for (yo, row) in (0..CHUNK_SIZE.y).zip(&height_data.0) {
            for (xo, height) in (0..CHUNK_SIZE.x).zip(row) {
                let pos = world_pos.0 + UVec2::new(xo, yo).as_ivec2();

                let mut height = *height;

                let distance_from_centre = pos.length_squared();
                if distance_from_centre < TERRAIN_STARTING_RADIUS.pow(2) {
                    // Ensure the starting zone isn't water by biasing towards grass

                    height = lerp_f32(
                        0.5,
                        height,
                        distance_from_centre.isqrt() as f32 / TERRAIN_STARTING_RADIUS as f32,
                    );
                }

                let tile = match height {
                    -1_f32..0. => TerrainSprite::Water,
                    0_f32..=1. => TerrainSprite::Grass,
                    _ => unreachable!("Generated sample with value: {height}"),
                };

                terrain_data.0[yo as usize][xo as usize] = tile;
            }
        }

        // Calculate local gradient with sobel filter
        // TODO: Proper handling of edges
        let mut gradient_data = GradientData::default();
        let h = &height_data.0;
        for i in 0..CHUNK_SIZE.y as usize {
            for j in 0..CHUNK_SIZE.x as usize {
                // Unpack values
                let h00 = if i == 0 || j == 0 {
                    0.
                } else {
                    h[i - 1][j - 1]
                };
                let h01 = if i == 0 { 0. } else { h[i - 1][j] };
                let h02 = if i == 0 || j == CHUNK_SIZE.x as usize - 1 {
                    0.
                } else {
                    h[i - 1][j + 1]
                };
                let h10 = if j == 0 { 0. } else { h[i][j - 1] };
                let h12 = if j == CHUNK_SIZE.x as usize - 1 {
                    0.
                } else {
                    h[i][j + 1]
                };

                let h20 = if i == CHUNK_SIZE.y as usize - 1 || j == 0 {
                    0.
                } else {
                    h[i + 1][j - 1]
                };
                let h21 = if i == CHUNK_SIZE.y as usize - 1 {
                    0.
                } else {
                    h[i + 1][j]
                };
                let h22 = if i == CHUNK_SIZE.y as usize - 1 || j == CHUNK_SIZE.x as usize - 1 {
                    0.
                } else {
                    h[i + 1][j + 1]
                };

                // Compute axis gradients
                let gy = (h20 + 2. * h21 + h22) - (h00 + 2. * h01 + h02);
                let gx = (h02 + 2. * h12 + h22) - (h00 + 2. * h10 + h20);

                gradient_data.0[i][j] = Vec2::new(gx, gy);
            }
        }

        (height_data, terrain_data, gradient_data)
    }
}

/// Lookup table for Chunk entities
#[derive(Resource, Default)]
pub struct ChunkLUT(pub HashMap<ChunkPos, Entity>);

/// Message which triggers a chunk to be created
#[derive(Message)]
pub struct CreateChunk(pub ChunkPos);

/// Event emitted after a chunk is created
#[derive(EntityEvent)]
pub struct ChunkCreated(pub Entity);
