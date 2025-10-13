use bevy::{platform::collections::HashMap, prelude::*, sprite_render::TilemapChunkTileData};
use rand::random_bool;

use crate::{
    consts::{
        CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG, RESOURCE_SPAWN_AMOUNT, Z_RESOURCES,
    },
    map::{ChunkCreated, ChunkLUT, TilePos},
    player::Targettable,
    sprites::{ResourceSprite, SpriteSheets, TerrainSprite},
    utils,
};

pub struct ResourcePlugin;
impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceNodes>()
            .add_observer(spawn_resources);
    }
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ResourceType {
    Wood,
    Food,
    Water,
}

/// The amount of resource left in a node
#[derive(Component)]
pub struct ResourceAmount(pub usize);

/// Sparse lookup for all resource nod entities spawned in the world
#[derive(Resource, Default)]
struct ResourceNodes(HashMap<TilePos, Entity>);

#[derive(Component)]
pub struct ResourceMarker;
/// Populate a chunk with naturally spawning resources
fn spawn_resources(
    event: On<ChunkCreated>,
    mut commands: Commands,
    mut resources: ResMut<ResourceNodes>,
    sprite_sheet: Res<SpriteSheets>,
    chunk_lut: Res<ChunkLUT>,
    tile_data: Query<&TilemapChunkTileData>,
) {
    let choices = [
        (ResourceSprite::Log, ResourceType::Wood),
        (ResourceSprite::Bush, ResourceType::Food),
    ];
    let weights = [RESOURCE_DENSITY_LOG, RESOURCE_DENSITY_BUSH];
    let total_weight = weights.iter().sum::<f32>().min(1.) as f64;

    let chunk_tile_pos = event.0.as_tile_pos();

    let tile_data = tile_data
        .get(chunk_lut.0[&event.0])
        .expect("Chunk entity should exist at this point");

    info!(
        "Spawning resources for chunk: {:?}, pos {:?}",
        event.0.0, chunk_tile_pos.0
    );
    for y in 0..CHUNK_SIZE.y {
        for x in 0..CHUNK_SIZE.x {
            if random_bool(total_weight) {
                // Check what terrain is on this tile
                // TODO: Create data layer in terrain so we don't need to work with TileChunk
                // data directly
                let terrain = tile_data[((CHUNK_SIZE.x - y - 1) * CHUNK_SIZE.x + x) as usize];
                if terrain
                    .map(|t| {
                        TerrainSprite::try_from(t.tileset_index as usize)
                            .expect("Invalid index for TerrainSprite Enum")
                    })
                    .is_none_or(|t| t != TerrainSprite::Grass)
                {
                    // Resources can only spawn on grass
                    continue;
                }

                let tile_pos = TilePos(chunk_tile_pos.0 + IVec2::new(x as i32, y as i32));

                let (sprite, res_type) = *utils::rand::choice(&choices, &weights);
                let entity = commands.spawn((
                    // Game data
                    tile_pos,
                    res_type,
                    // TODO: Resource amount spawn logic
                    ResourceAmount(RESOURCE_SPAWN_AMOUNT),
                    ResourceMarker,
                    Targettable,
                    // Render
                    Sprite {
                        image: sprite_sheet.resources.image.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: sprite_sheet.resources.layout.clone(),
                            index: sprite as usize,
                        }),
                        ..Default::default()
                    },
                    tile_pos.as_transform(Z_RESOURCES),
                ));

                resources.0.insert(tile_pos, entity.id());
            }
        }
    }
}
