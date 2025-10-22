use super::components::*;
use bevy::prelude::*;
use rand::random_bool;

use crate::{
    consts::{
        CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG, RESOURCE_SPAWN_AMOUNT,
        RESOURCE_STARTING_RADIUS, Z_RESOURCES,
    },
    items::ItemType,
    map::{ChunkCreated, ChunkPos, TerrainData, TilePos},
    player::Targettable,
    sprites::{GetSprite, SpriteSheets, TerrainSprite},
    utils,
};

/// Populate a chunk with naturally spawning resources
pub fn spawn_resources(
    event: On<ChunkCreated>,
    mut commands: Commands,
    mut resources: ResMut<ResourceNodes>,
    sprite_sheets: Res<SpriteSheets>,
    chunks: Query<(&ChunkPos, &TerrainData)>,
) {
    let choices = [
        (ResourceNodeType::Tree, ItemType::Log),
        (ResourceNodeType::Bush, ItemType::Berry),
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

                let (node_type, item_type) = *utils::rand::choice(&choices, &weights);
                let entity = commands
                    .spawn((
                        // Game data
                        tile_pos,
                        item_type,
                        node_type,
                        // TODO: Resource amount spawn logic
                        ResourceAmount(RESOURCE_SPAWN_AMOUNT),
                        ResourceMarker,
                        Targettable,
                        // Render
                        tile_pos.as_transform(Z_RESOURCES),
                    ))
                    .id();

                // Add sprite as child
                node_type.spawn_sprite(&mut commands, &sprite_sheets, Some(entity));

                resources.0.insert(tile_pos, entity);
            }
        }
    }
}
