use bevy::{prelude::*, sprite_render::TilemapChunk};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::consts::TILE_RAW_SIZE;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        // Resource loading before any game stuff
        app.add_systems(PreStartup, load_sprite_sheets)
            .add_systems(Update, tilemap_post_load);
    }
}

/// Indexes into terrain_sprites.png
#[derive(Component, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum TerrainSprite {
    #[default]
    Grass,
    Water,
    Blank,
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
            // Assume vertically stacked, same sized sprites
            let depth = image.height() / image.width();
            image.reinterpret_stacked_2d_as_array(depth);
        }
    }
}

/// Indexes into resource_sprites.png
#[derive(Component, Clone, Copy)]
#[repr(usize)]
pub enum ResourceSprite {
    Log,
    Bush,
    House,
}

/// entity_sheet.png
#[derive(Component, Clone, Copy)]
#[repr(usize)]
pub enum EntitySprite {
    Player,
}

pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub struct SpriteSheets {
    /// Resource_sheet.png
    pub resources: SpriteSheet,
    /// entity_sheet.png
    pub entities: SpriteSheet,
}

fn load_sprite_sheets(
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

    commands.insert_resource(SpriteSheets {
        resources,
        entities,
    });
}
