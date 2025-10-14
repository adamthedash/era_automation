use bevy::{
    platform::collections::HashMap,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{
    consts::{CHUNK_SIZE, TERRAIN_STARTING_RADIUS, TILE_DISPLAY_SIZE, TILE_RAW_SIZE},
    sprites::TerrainSprite,
    utils::{
        math::lerp,
        noise::{MyGenerator, perlin_stack},
    },
};

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_world_gen)
            .add_systems(
                Update,
                (create_chunks, update_transforms, update_tilemap_data),
            )
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

    pub fn to_chunk_offset(&self) -> (ChunkPos, UVec2) {
        let chunk = ChunkPos(self.0.div_euclid(CHUNK_SIZE.as_ivec2()));
        let offset = self.0.rem_euclid(CHUNK_SIZE.as_ivec2()).as_uvec2();

        (chunk, offset)
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

#[derive(Component, Clone, Copy)]
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

/// Game data friendly storage for terrain tiles
#[derive(Component, Default)]
pub struct TerrainData(pub [[TerrainSprite; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize]);

/// Random generation for everything in the world
#[derive(Resource)]
pub struct WorldGenerator {
    pub terrain: Box<dyn MyGenerator<2>>,
}
impl WorldGenerator {
    fn generate_terrain(&self, pos: ChunkPos) -> TerrainData {
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

/// Set up the world generation
pub fn init_world_gen(mut commands: Commands) {
    commands.insert_resource(WorldGenerator {
        terrain: Box::new(perlin_stack(42, 4, 1., 0.5, 1. / 16., 0.)),
    });
}

#[derive(Resource, Default)]
pub struct ChunkLUT(pub HashMap<ChunkPos, Entity>);

/// Message which triggers a chunk to be created
#[derive(Message)]
pub struct CreateChunk(pub ChunkPos);

/// Event emitted after a chunk is created
#[derive(EntityEvent)]
pub struct ChunkCreated(Entity);

/// Initialises a new chunk at a given position
pub fn create_chunks(
    mut messages: MessageReader<CreateChunk>,
    mut commands: Commands,
    mut chunk_lut: ResMut<ChunkLUT>,
    asset_server: Res<AssetServer>,
    generator: Res<WorldGenerator>,
) {
    for CreateChunk(pos) in messages.read() {
        info!("Spawning chunk entity: {:?}", pos.0);
        let mut chunk = commands.spawn((
            *pos,
            Transform::from_translation(
                // +0.5 chunks so TileMapChunk is rendered from its origin
                // -0.5 tiles so resource sprites are aligned properly
                (((pos.0.as_vec2() + Vec2::splat(0.5)) * CHUNK_SIZE.as_vec2() - Vec2::splat(0.5))
                    * TILE_DISPLAY_SIZE.as_vec2())
                .extend(0.),
            ),
            Visibility::default(),
            // Terrain data
            generator.generate_terrain(*pos),
            // Render
            TilemapChunk {
                chunk_size: CHUNK_SIZE,
                tile_display_size: TILE_DISPLAY_SIZE,
                tileset: asset_server.load("terrain_sheet.png"),
                ..Default::default()
            },
            TilemapChunkTileData(vec![None; CHUNK_SIZE.element_product() as usize]),
        ));

        chunk_lut.0.insert(*pos, chunk.id());
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
