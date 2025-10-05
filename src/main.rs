use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

fn main() {
    App::new()
        //
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(First, tilemap_post_load)
        //
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let chunk_size = UVec2::splat(16);
    let tile_display_size = UVec2::splat(128);
    let tile_data =
        vec![Some(TileData::from_tileset_index(0)); chunk_size.element_product() as usize];

    commands.spawn((
        TilemapChunk {
            chunk_size,
            tile_display_size,
            tileset: asset_server.load("sprite_sheet.png"),
            ..Default::default()
        },
        TilemapChunkTileData(tile_data),
    ));
}

/// After loading the sprite sheet, it must be turned into a 2d image array so the images can be
/// indexed into properly. Not sure why this needs to be ran on an update schedule, and can't be
/// baked into when the tilemap is spawned in.
fn tilemap_post_load(
    chunk_query: Single<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let chunk = *chunk_query;
    for event in events.read() {
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image.reinterpret_stacked_2d_as_array(2);
        }
    }
}
