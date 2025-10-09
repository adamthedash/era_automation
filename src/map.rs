use bevy::{
    platform::collections::HashMap,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{
    consts::{CHUNK_SIZE, TILE_DISPLAY_SIZE, TILE_RAW_SIZE},
    sprites::TerrainSprite,
};

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_chunks)
            .init_resource::<ChunkLUT>()
            .add_message::<CreateChunk>();
    }
}

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
}

#[derive(Component, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkPos(pub IVec2);
impl ChunkPos {
    /// Convert from display-space transform
    pub fn from_transform(transform: &Transform) -> Self {
        let world_pos = WorldPos::from_transform(transform);
        Self(world_pos.0.as_ivec2() / CHUNK_SIZE.as_ivec2())
    }

    pub fn as_tile_pos(&self) -> TilePos {
        TilePos(self.0 * CHUNK_SIZE.as_ivec2())
    }
}

pub struct WorldPos(pub Vec2);
impl WorldPos {
    pub fn from_transform(transform: &Transform) -> Self {
        Self(transform.translation.truncate() / TILE_DISPLAY_SIZE.as_vec2())
    }

    pub fn as_transform(&self, z: f32) -> Transform {
        Transform::from_translation((self.0 * TILE_DISPLAY_SIZE.as_vec2()).extend(z))
            .with_scale((TILE_DISPLAY_SIZE.as_vec2() / TILE_RAW_SIZE.as_vec2()).extend(1.))
    }
}

#[derive(Resource, Default)]
pub struct ChunkLUT(pub HashMap<ChunkPos, Entity>);

/// Message which triggers a chunk to be created
#[derive(Message)]
pub struct CreateChunk(pub ChunkPos);

/// Event emitted after a chunk is created
#[derive(Event)]
pub struct ChunkCreated(pub ChunkPos);

/// Initialises a new chunk at a given position
fn create_chunks(
    mut messages: MessageReader<CreateChunk>,
    mut commands: Commands,
    mut chunk_lut: ResMut<ChunkLUT>,
    asset_server: Res<AssetServer>,
) {
    for CreateChunk(pos) in messages.read() {
        let chunk = commands.spawn((
            *pos,
            Transform::from_translation(
                (pos.0.as_vec2() * CHUNK_SIZE.as_vec2() * TILE_DISPLAY_SIZE.as_vec2()).extend(0.),
            ),
            Visibility::default(),
            // Terrain data
            TilemapChunk {
                chunk_size: CHUNK_SIZE,
                tile_display_size: TILE_DISPLAY_SIZE,
                tileset: asset_server.load("terrain_sheet.png"),
                ..Default::default()
            },
            TilemapChunkTileData(vec![
                Some(TileData::from_tileset_index(
                    TerrainSprite::Grass as u16
                ));
                CHUNK_SIZE.element_product() as usize
            ]),
        ));

        chunk_lut.0.insert(*pos, chunk.id());
        commands.trigger(ChunkCreated(*pos));
    }
}
