use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    consts::{CHUNK_SIZE, TERRAIN_STARTING_RADIUS},
    sprites::TerrainSprite,
    utils::{math::lerp, noise::MyGenerator},
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

/// Random generation for everything in the world
#[derive(Resource)]
pub struct WorldGenerator {
    pub terrain: Box<dyn MyGenerator<2>>,
}
impl WorldGenerator {
    pub fn generate_terrain(&self, pos: ChunkPos) -> TerrainData {
        let mut data = TerrainData::default();

        let world_pos = pos.as_tile_pos();
        for yo in 0..CHUNK_SIZE.y {
            for xo in 0..CHUNK_SIZE.x {
                let pos = world_pos.0 + UVec2::new(xo, yo).as_ivec2();

                let mut sample = self.terrain.sample([pos.x as f64, pos.y as f64]);

                let distance_from_centre = pos.length_squared();
                if distance_from_centre < TERRAIN_STARTING_RADIUS.pow(2) {
                    // Ensure the starting zone isn't water by biasing towards grass

                    sample = lerp(
                        1.,
                        sample,
                        distance_from_centre.isqrt() as f64 / TERRAIN_STARTING_RADIUS as f64,
                    );
                }

                let tile = match sample {
                    -1_f64..0. => TerrainSprite::Water,
                    0_f64..=1. => TerrainSprite::Grass,
                    _ => unreachable!("Generated sample with value: {sample}"),
                };

                data.0[yo as usize][xo as usize] = tile;
            }
        }

        data
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
