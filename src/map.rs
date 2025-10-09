use bevy::{
    platform::collections::HashMap,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{
    SpriteSheet,
    consts::{CHUNK_SIZE, TILE_DISPLAY_SIZE},
    sprites::TerrainSprite,
};

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tilemap_post_load, create_chunks))
            .init_resource::<ChunkLUT>()
            .add_message::<CreateChunk>();
    }
}

#[derive(Component, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TilePos(pub IVec2);
impl TilePos {
    pub fn as_transform(&self, z: f32) -> Transform {
        Transform::from_translation(self.0.as_vec2().extend(z))
    }
}

#[derive(Component, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkPos(pub IVec2);
impl ChunkPos {
    pub fn from_transform(transform: &Transform) -> Self {
        Self(
            transform
                .translation
                .truncate()
                .as_ivec2()
                .div_euclid(CHUNK_SIZE.as_ivec2()),
        )
    }

    pub fn as_tile_pos(&self) -> TilePos {
        TilePos(self.0 * CHUNK_SIZE.as_ivec2())
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
                tileset: asset_server.load("terrain_sheet.png#terrain"),
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

/// After loading the sprite sheet, it must be turned into a 2d image array so the images can be
/// indexed into properly. Not sure why this needs to be ran on an update schedule, and can't be
/// baked into when the tilemap is spawned in.
fn tilemap_post_load(
    chunk_query: Query<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(chunk) = chunk_query.iter().next() else {
        // No chunks generated yet
        return;
    };

    for event in events.read() {
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image
                .reinterpret_stacked_2d_as_array(std::mem::variant_count::<TerrainSprite>() as u32);
        }
    }
}
