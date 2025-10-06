mod consts;
mod sprites;

use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::random_range;

use crate::consts::{CAMERA_SPEED, CHUNK_SIZE, NUM_SPRITES, TILE_DISPLAY_SIZE};

fn main() {
    App::new()
        //
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup_map, setup_player))
        .add_systems(Update, (tilemap_post_load, move_player))
        //
        .run();
}

/// Spawn in the tiles for the world
#[derive(Component)]
struct TerrainLayer;
#[derive(Component)]
struct ResourceLayer;
fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let terrain_data =
        vec![Some(TileData::from_tileset_index(0)); CHUNK_SIZE.element_product() as usize];

    for x in -3..3 {
        for y in -3..3 {
            // Terrain
            commands.spawn((
                TilemapChunk {
                    chunk_size: CHUNK_SIZE,
                    tile_display_size: TILE_DISPLAY_SIZE,
                    tileset: asset_server.load("terrain_sheet.png"),
                    ..Default::default()
                },
                TilemapChunkTileData(terrain_data.clone()),
                Transform::from_translation(
                    (Vec2::new(x as f32, y as f32)
                        * CHUNK_SIZE.as_vec2()
                        * TILE_DISPLAY_SIZE.as_vec2())
                    // Z = 0 for terrain
                    .extend(0.),
                ),
                TerrainLayer,
            ));

            // Resources
            let mut resource_data = vec![None; CHUNK_SIZE.element_product() as usize];
            for _ in 0..10 {
                // Trees
                let i = random_range(0..resource_data.len());
                resource_data[i] = Some(TileData::from_tileset_index(1));

                // Bushes
                let i = random_range(0..resource_data.len());
                resource_data[i] = Some(TileData::from_tileset_index(2));
            }

            commands.spawn((
                TilemapChunk {
                    chunk_size: CHUNK_SIZE,
                    tile_display_size: TILE_DISPLAY_SIZE,
                    tileset: asset_server.load("terrain_sheet.png"),
                    ..Default::default()
                },
                TilemapChunkTileData(resource_data),
                Transform::from_translation(
                    (Vec2::new(x as f32, y as f32)
                        * CHUNK_SIZE.as_vec2()
                        * TILE_DISPLAY_SIZE.as_vec2())
                    // Z = 1 for resources
                    .extend(1.),
                ),
                ResourceLayer,
            ));
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
            image.reinterpret_stacked_2d_as_array(NUM_SPRITES);
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
