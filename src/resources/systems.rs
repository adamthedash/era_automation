use bevy::prelude::*;
use rand::random_bool;

use super::{bundles::*, components::*};
use crate::{
    consts::{
        CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG, RESOURCE_MAX_AMOUNT,
        RESOURCE_PICKUP_AMOUNT, RESOURCE_REGEN_RATE, RESOURCE_STARTING_RADIUS, Z_RESOURCES,
    },
    items::ItemType,
    map::{ChunkCreated, ChunkPos, TerrainData},
    player::Targettable,
    sprites::{GetSprite, ResourceSprite, SpriteSheets, TerrainSprite},
    utils,
};

/// Populate a chunk with naturally spawning resources
pub fn spawn_resources(
    event: On<ChunkCreated>,
    mut commands: Commands,
    mut resources: ResMut<ResourceNodeLUT>,
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
                let tile_pos = chunk_tile_pos + IVec2::new(x as i32, y as i32);
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
                        // TODO: Resource amount spawn logic
                        ResourceNodeBundle::new(
                            node_type,
                            RESOURCE_PICKUP_AMOUNT,
                            RESOURCE_MAX_AMOUNT,
                            RESOURCE_REGEN_RATE,
                        ),
                        // For now, nodes spawn full
                        ResourceNodeFull,
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

/// Sync resource node sprites when their `ResourceAmount` changes.
pub fn sync_resource_sprites(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    query: Query<
        (Entity, &ResourceNodeType, &ResourceAmount, &Children),
        (With<ResourceMarker>, Changed<ResourceAmount>),
    >,
    sprite_entities: Query<(), With<Sprite>>,
) {
    for (entity, node_type, amount, children) in query.iter() {
        // Despawn old sprite
        let sprite = children
            .iter()
            .find(|child| sprite_entities.get(*child).is_ok())
            .expect("Node has no sprite!");
        commands.entity(sprite).despawn();

        // Choose the appropriate sprite based on remaining amount
        let sprite_to_spawn = if amount.0 == 0 {
            match node_type {
                ResourceNodeType::Tree => ResourceSprite::TreeDepleted,
                ResourceNodeType::Bush => ResourceSprite::BushDepleted,
                ResourceNodeType::Water => unreachable!("Water node should never be rendered"),
            }
        } else {
            match node_type {
                ResourceNodeType::Tree => ResourceSprite::Tree,
                ResourceNodeType::Bush => ResourceSprite::Bush,
                ResourceNodeType::Water => unreachable!("Water node should never be rendered"),
            }
        };

        // Spawn the selected sprite as a child of the resource entity
        sprite_to_spawn.spawn_sprite(&mut commands, &sprite_sheets, Some(entity));
    }
}

/// Tick resource nodes to regenerate resources
pub fn regenerate_resource_nodes(
    resource_nodes: Query<
        (
            &ResourceRegenRate,
            &mut ResourceRegenState,
            &mut ResourceAmount,
            &ResourceMaxAmount,
        ),
        (With<ResourceMarker>, Without<ResourceNodeFull>),
    >,
    timer: Res<Time>,
) {
    for (rate, mut state, mut amount, max) in resource_nodes {
        // Grow
        state.0 += rate.0 * timer.delta_secs();
        while state.0 >= 1. && amount.0 < max.0 {
            amount.0 += 1;
            state.0 -= 1.;
        }
    }
}

/// Adds / removes the full marker so that resources aren't constantly ticked
pub fn mark_resource_full(
    resource_nodes: Query<
        (
            Entity,
            &ResourceAmount,
            &ResourceMaxAmount,
            Has<ResourceNodeFull>,
        ),
        (With<ResourceMarker>, Changed<ResourceAmount>),
    >,
    mut commands: Commands,
) {
    for (entity, amount, max, has_marker) in resource_nodes {
        if amount.0 >= max.0 && !has_marker {
            // Just became full
            commands.entity(entity).insert(ResourceNodeFull);
        } else if amount.0 < max.0 && has_marker {
            // Just became not full
            commands.entity(entity).remove::<ResourceNodeFull>();
        }
    }
}
