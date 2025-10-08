#![feature(variant_count)]
mod consts;
mod sprites;
mod utils;
mod village;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::random_bool;

use crate::{
    consts::{
        CAMERA_SPEED, CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG, TILE_DISPLAY_SIZE,
    },
    sprites::TerrainSprite,
    village::{VillagePlugin, setup_village},
};

fn main() {
    App::new()
        //
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(VillagePlugin)
        .init_resource::<Chunks>()
        .add_systems(
            Startup,
            (
                (setup_map, spawn_village, spawn_resources).chain(),
                setup_player,
            ),
        )
        .add_systems(
            Update,
            (tilemap_post_load, move_player, update_resource_sprites),
        )
        //
        .run();
}

/// Entity handles for resources in each tile
#[derive(Component, Default)]
struct ResourceData([[Option<Entity>; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize]);
impl ResourceData {
    fn width(&self) -> usize {
        self.0[0].len()
    }

    fn height(&self) -> usize {
        self.0.len()
    }
}

#[derive(Component, Hash, PartialEq, Eq, Clone)]
struct ChunkPos(IVec2);
#[derive(Resource, Default)]
struct Chunks(HashMap<ChunkPos, Entity>);
impl Chunks {
    /// Get the chunk entity and offset within for a given tile position
    fn chunk_for_tile(&self, tile_pos: IVec2) -> (Entity, UVec2) {
        let chunk_pos = ChunkPos(IVec2 {
            x: tile_pos.x / CHUNK_SIZE.x as i32,
            y: tile_pos.y / CHUNK_SIZE.y as i32,
        });
        let offset = IVec2 {
            x: tile_pos.x % CHUNK_SIZE.x as i32,
            y: tile_pos.y % CHUNK_SIZE.y as i32,
        }
        .as_uvec2();

        let chunk = self.0.get(&chunk_pos).expect("Chunk not loaded");

        (*chunk, offset)
    }
}

#[derive(Component)]
struct TerrainLayer;
#[derive(Component)]
struct MachineryLayer;
#[derive(Component)]
struct ResourceLayer;

/// Set up the tilemap rendering for the world
fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>, mut chunks: ResMut<Chunks>) {
    let terrain_data = vec![
        Some(TileData::from_tileset_index(TerrainSprite::Grass as u16));
        CHUNK_SIZE.element_product() as usize
    ];

    for x in -3..3 {
        for y in -3..3 {
            let chunk_pos = ChunkPos(IVec2::new(x, y));
            let mut entity = commands.spawn((
                chunk_pos.clone(),
                Transform::from_translation(
                    (Vec2::new(x as f32, y as f32)
                        * CHUNK_SIZE.as_vec2()
                        * TILE_DISPLAY_SIZE.as_vec2())
                    .extend(0.),
                ),
                Visibility::default(),
            ));
            entity.with_children(|spawner| {
                // Terrain
                spawner.spawn((
                    TilemapChunk {
                        chunk_size: CHUNK_SIZE,
                        tile_display_size: TILE_DISPLAY_SIZE,
                        tileset: asset_server.load("terrain_sheet.png"),
                        ..Default::default()
                    },
                    TilemapChunkTileData(terrain_data.clone()),
                    // Z = 0 for terrain
                    Transform::from_translation(Vec3::Z * 0.),
                    TerrainLayer,
                ));

                // Machinery
                spawner.spawn((
                    TilemapChunk {
                        chunk_size: CHUNK_SIZE,
                        tile_display_size: TILE_DISPLAY_SIZE,
                        tileset: asset_server.load("terrain_sheet.png"),
                        ..Default::default()
                    },
                    TilemapChunkTileData(vec![None; CHUNK_SIZE.element_product() as usize]),
                    // Z = 1 for machinery
                    Transform::from_translation(Vec3::Z * 1.),
                    MachineryLayer,
                ));

                // Resources
                spawner.spawn((
                    TilemapChunk {
                        chunk_size: CHUNK_SIZE,
                        tile_display_size: TILE_DISPLAY_SIZE,
                        tileset: asset_server.load("terrain_sheet.png"),
                        ..Default::default()
                    },
                    TilemapChunkTileData(vec![None; CHUNK_SIZE.element_product() as usize]),
                    ResourceData::default(),
                    // Z = 1 for resources
                    Transform::from_translation(Vec3::Z * 1.),
                    ResourceLayer,
                ));
            });

            chunks.0.insert(chunk_pos, entity.id());
        }
    }
}

#[derive(Component)]
struct Player;
fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Sprite::from_color(Color::WHITE, TILE_DISPLAY_SIZE.as_vec2()),
        Player,
        // Z == 2 for player
        Transform::from_xyz(0., 0., 2.),
    ));
}

/// After loading the sprite sheet, it must be turned into a 2d image array so the images can be
/// indexed into properly. Not sure why this needs to be ran on an update schedule, and can't be
/// baked into when the tilemap is spawned in.
fn tilemap_post_load(
    chunk_query: Query<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let chunk = chunk_query.iter().next().unwrap();
    for event in events.read() {
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image
                .reinterpret_stacked_2d_as_array(std::mem::variant_count::<TerrainSprite>() as u32);
        }
    }
}

fn move_player(
    mut player: Single<&mut Transform, With<Player>>,
    inputs: Res<ButtonInput<KeyCode>>,
    timer: Res<Time>,
) {
    let x = match (inputs.pressed(KeyCode::KeyA), inputs.pressed(KeyCode::KeyD)) {
        (true, false) => -1.,
        (false, true) => 1.,
        _ => 0.,
    };
    let y = match (inputs.pressed(KeyCode::KeyS), inputs.pressed(KeyCode::KeyW)) {
        (true, false) => -1.,
        (false, true) => 1.,
        _ => 0.,
    };

    player.translation += (x * Vec3::X + y * Vec3::Y) * CAMERA_SPEED * timer.delta_secs();
}

/// Random generation for resources
fn spawn_resources(
    mut commands: Commands,
    resources: Query<&mut ResourceData, With<ResourceLayer>>,
) {
    let choices = [TerrainSprite::Log, TerrainSprite::Bush];
    let weights = [RESOURCE_DENSITY_LOG, RESOURCE_DENSITY_BUSH];
    let total_weight = weights.iter().sum::<f32>().min(1.) as f64;
    for mut resource_data in resources {
        for i in 0..resource_data.height() {
            for j in 0..resource_data.width() {
                if random_bool(total_weight) {
                    // Spawn entity
                    let entity = match utils::rand::choice(&choices, &weights) {
                        TerrainSprite::Log => commands.spawn(TerrainSprite::Log),
                        TerrainSprite::Bush => commands.spawn(TerrainSprite::Bush),
                        _ => unreachable!(),
                    };

                    // Add to data grid
                    resource_data.0[i][j] = Some(entity.id());
                }
            }
        }
    }
}

/// Sync resource tilemap with underlying data
fn update_resource_sprites(
    query: Query<
        (&mut TilemapChunkTileData, &ResourceData),
        (With<ResourceLayer>, Changed<ResourceData>),
    >,
    resources: Query<&TerrainSprite>,
) {
    for (mut dst, src) in query {
        for (i, row) in src.0.iter().enumerate() {
            for (j, entity) in row.iter().enumerate() {
                dst[i * row.len() + j] = entity.map(|entity| {
                    let sprite = resources.get(entity).expect("Resource entity not found");

                    TileData::from_tileset_index(*sprite as u16)
                })
            }
        }
    }
}

fn spawn_village(
    chunks: Query<&Children, With<ChunkPos>>,
    mut machine_layers: Query<&mut TilemapChunkTileData, With<MachineryLayer>>,
    chunk_lut: Res<Chunks>,
) {
    let (chunk, offset) = chunk_lut.chunk_for_tile(IVec2::ZERO);
    let children = chunks.get(chunk).expect("Chunk not generated");
    let machine_entity = children
        .iter()
        .find(|entity| machine_layers.get_mut(*entity).is_ok())
        .expect("MachineryLayer doesn't exist");
    let mut data = machine_layers
        .get_mut(machine_entity)
        .expect("MachineryLayer doesn't exist");

    data[(offset.y * CHUNK_SIZE.y + offset.x) as usize] =
        Some(TileData::from_tileset_index(TerrainSprite::House as u16));
}
