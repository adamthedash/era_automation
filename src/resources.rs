use bevy::{platform::collections::HashMap, prelude::*};
use rand::random_bool;

use crate::{
    consts::{
        CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG, RESOURCE_SPAWN_AMOUNT,
        RESOURCE_STARTING_RADIUS, Z_RESOURCES,
    },
    map::{ChunkCreated, ChunkPos, TerrainData, TilePos},
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

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ResourceType {
    Wood,
    Food,
    Water,
    /// TODO: Move this to dedicated items enum
    Bowl,
}

impl ResourceType {
    /// Get the corresponding sprite for this resource
    pub fn sprite(&self) -> ResourceSprite {
        match self {
            ResourceType::Wood => ResourceSprite::Log,
            ResourceType::Food => ResourceSprite::Bush,
            ResourceType::Water => ResourceSprite::Water,
            ResourceType::Bowl => ResourceSprite::Bowl,
        }
    }
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
    chunks: Query<(&ChunkPos, &TerrainData)>,
) {
    let choices = [
        (ResourceSprite::Log, ResourceType::Wood),
        (ResourceSprite::Bush, ResourceType::Food),
    ];
    let weights = [RESOURCE_DENSITY_LOG, RESOURCE_DENSITY_BUSH];
    let total_weight = weights.iter().sum::<f32>().min(1.) as f64;

    let (chunk_pos, tile_data) = chunks
        .get(event.event_target())
        .expect("Chunk entity should exist at this point");

    let chunk_tile_pos = chunk_pos.as_tile_pos();

    info!("Spawning resources for chunk: {:?}", chunk_pos.0);
    for y in 0..CHUNK_SIZE.y {
        for x in 0..CHUNK_SIZE.x {
            if random_bool(total_weight) {
                let tile_pos = TilePos(chunk_tile_pos.0 + IVec2::new(x as i32, y as i32));
                if tile_pos.0.length_squared() <= RESOURCE_STARTING_RADIUS.pow(2) {
                    // Resources can't spawn too close to the starting point
                    continue;
                }

                if tile_data.0[y as usize][x as usize] != TerrainSprite::Grass {
                    // Resources can only spawn on grass
                    continue;
                }

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
