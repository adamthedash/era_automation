use bevy::{prelude::*, sprite_render::TilemapChunk};

use super::components::*;
use crate::consts::TILE_RAW_SIZE;

/// After loading the sprite sheet, it must be turned into a 2d image array so the images can be
/// indexed into properly. Not sure why this needs to be ran on an update schedule, and can't be
/// baked into when the tilemap is spawned in.
pub fn tilemap_post_load(
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
            // Assume vertically stacked, same sized sprites
            let depth = image.height() / image.width();
            image.reinterpret_stacked_2d_as_array(depth);
        }
    }
}

/// Load up all of the sprite sheets
pub fn load_sprite_sheets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("resource_sheet.png");
    let layout = TextureAtlasLayout::from_grid(
        TILE_RAW_SIZE,
        1,
        std::mem::variant_count::<ResourceSprite>() as u32,
        None,
        None,
    );
    let layout = layouts.add(layout);
    let resources = SpriteSheet { image, layout };

    let image = asset_server.load("entity_sheet.png");
    let layout = TextureAtlasLayout::from_grid(
        TILE_RAW_SIZE,
        1,
        std::mem::variant_count::<EntitySprite>() as u32,
        None,
        None,
    );
    let layout = layouts.add(layout);
    let entities = SpriteSheet { image, layout };

    let image = asset_server.load("item_sheet.png");
    let layout = TextureAtlasLayout::from_grid(
        TILE_RAW_SIZE,
        1,
        std::mem::variant_count::<ItemSprite>() as u32,
        None,
        None,
    );
    let layout = layouts.add(layout);
    let items = SpriteSheet { image, layout };

    commands.insert_resource(SpriteSheets {
        resources,
        entities,
        items,
    });
}
