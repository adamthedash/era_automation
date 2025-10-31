use crate::map::Chunks;
use bevy::{
    self,
    ecs::{bundle::InsertMode, system::entity_command},
    prelude::*,
};

use super::components::*;
use crate::{
    consts::{
        CAMERA_ZOOM, HIGHLIGHT_SCALE, PLAYER_REACH, PLAYER_SPEED, RESOURCE_PICKUP_AMOUNT,
        Z_HELD_ITEM, Z_PLAYER,
    },
    container::{ContainableItems, ContainedBundle},
    items::ItemType,
    map::{TerrainData, TilePos, WorldPos},
    resources::{ResourceAmount, ResourceMarker, ResourceNodeType},
    sprites::{EntitySprite, GetSprite, ItemSprite, SpriteSheets, TerrainSprite},
};

pub fn setup_player(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
    let world_pos = WorldPos(Vec2::ZERO);
    commands.spawn((
        Camera2d,
        world_pos,
        Transform::from_scale(Vec2::splat(1. / CAMERA_ZOOM).extend(1.)),
    ));

    let player = commands
        .spawn((
            Player,
            world_pos,
            // Render
            world_pos.as_transform(Z_PLAYER),
        ))
        .id();

    EntitySprite::Player.spawn_sprite(&mut commands, &sprite_sheets, Some(player));
}

pub fn move_player(
    mut player: Single<&mut WorldPos, With<Player>>,
    mut camera: Single<&mut WorldPos, (With<Camera2d>, Without<Player>)>,
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
    if x == 0. && y == 0. {
        // Not moving
        return;
    }

    player.0 += Vec2::new(x, y) * PLAYER_SPEED * timer.delta_secs();

    // Update camera transform aswell
    camera.0 = player.0;
}

/// Targets the closest thing to the player
pub fn target_thing(
    mut commands: Commands,
    player: Single<&WorldPos, With<Player>>,
    targettables: Query<(Entity, Option<&TilePos>, Option<&WorldPos>), With<Targettable>>,
    targetted: Query<Entity, With<Targetted>>,
) {
    let closest = targettables
        .iter()
        .map(|(entity, tile_pos, world_pos)| {
            // Collapse position types
            let pos = world_pos.copied().unwrap_or_else(|| {
                tile_pos
                    .expect("Entity has neither tile or world position")
                    .as_world_pos()
            });

            // Calculate distance
            let distance2 = player.0.distance_squared(pos.0);

            (entity, pos, distance2)
        })
        .filter(|(_, _, distance2)| *distance2 <= PLAYER_REACH.powi(2))
        .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(d2));

    for entity in targetted {
        if let Some((closest_entity, _, _)) = closest
            && entity == closest_entity
        {
            // Same entity targetted, don't remove it
            continue;
        }

        commands
            .entity(entity)
            // Entity may be removed by the time this is ran, in which case it doesn't matter
            .queue_silenced(entity_command::remove::<Targetted>());
    }

    if let Some((entity, _, _)) = closest {
        commands
            .entity(entity)
            .queue_silenced(entity_command::insert(Targetted, InsertMode::Replace));
    }
}

/// Make targetted things bigger
pub fn highlight_target(
    event: On<Add, Targetted>,
    mut transforms: Query<(Entity, &mut Transform), With<Targetted>>,
) {
    if let Ok((entity, mut transform)) = transforms.get_mut(event.entity) {
        info!("Highlighting target {:?}", entity);
        transform.scale *= Vec2::splat(HIGHLIGHT_SCALE).extend(1.);
    }
}

/// Make untargetted things smaller
pub fn unhighlight_target(
    event: On<Remove, Targetted>,
    mut transforms: Query<(Entity, &mut Transform)>,
) {
    if let Ok((entity, mut transform)) = transforms.get_mut(event.entity) {
        info!("Un-highlighting target {:?}", entity);
        transform.scale /= Vec2::splat(HIGHLIGHT_SCALE).extend(1.);
    }
}

/// Triggered when something is made untargettable
pub fn make_untargettable(event: On<Remove, Targettable>, mut commands: Commands) {
    // Might be triggered by entity despawning, so try remove
    commands.entity(event.entity).try_remove::<Targetted>();
}

/// Pick up a resource and put it in the player's hand
pub fn harvest_resource(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    mut targetted_resources: Populated<
        (Entity, &ResourceNodeType, &ItemType, &mut ResourceAmount),
        (With<ResourceMarker>, With<Targetted>, Without<Player>),
    >,
    sprite_sheets: Res<SpriteSheets>,
) {
    let (resource_entity, node_type, item_type, mut amount) = targetted_resources
        .iter_mut()
        .next()
        .unwrap_or_else(|| unreachable!("Populated query"));

    info!("Harvesting resource: {:?}", node_type);

    let pickup_amount = RESOURCE_PICKUP_AMOUNT.min(amount.0);

    // Add item to player
    let entity = commands
        .spawn((
            HeldItemBundle::new(*player),
            // Game data
            *item_type,
        ))
        .id();

    item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(entity));

    // Remove resource if it's depleted
    if amount.0 == pickup_amount {
        // Player has grabbed it all, so remove the node
        commands.entity(resource_entity).despawn();
    } else {
        // Player has only picked up some of it
        amount.0 -= pickup_amount;
    }

    commands.trigger(HarvestEvent {
        resource_node: *node_type,
        amount: pickup_amount,
    });
}

/// Checks whether the player is in range of a water source
pub fn check_near_water(
    player: Single<(Entity, &WorldPos), With<Player>>,
    tile_data: Chunks<&TerrainData>,
    mut commands: Commands,
) {
    let (player_entity, player_pos) = *player;

    // Check nearby tiles to see if they're water
    let min = (player_pos.0 - PLAYER_REACH).floor().as_ivec2();
    let max = (player_pos.0 + PLAYER_REACH).ceil().as_ivec2();

    let near_water = (min.x..max.x)
        .flat_map(|x| (min.y..max.y).map(move |y| TilePos(IVec2::new(x, y))))
        // Tiles within reach
        .filter(|tile_pos| {
            let tile_centre = tile_pos.0.as_vec2() + 0.5;
            player_pos.0.distance_squared(tile_centre) <= PLAYER_REACH.powi(2)
        })
        // Fetch chunk data, skip if it's not been initalised yet
        .filter_map(|tile_pos| {
            let (chunk_pos, offset) = tile_pos.to_chunk_offset();

            // Fetch tile data for the chunk
            tile_data.get(&chunk_pos).map(|td| (offset, td))
        })
        // Check if any are water tiles
        .any(|(offset, tile_data)| {
            tile_data.0[offset.y as usize][offset.x as usize] == TerrainSprite::Water
        });

    if near_water {
        commands.entity(player_entity).insert_if_new(NearWater);
    } else {
        commands.entity(player_entity).remove::<NearWater>();
    }
}

/// Shows the water icon when the player is near the water
pub fn show_water_icon(
    player: Single<(Entity, Has<NearWater>, Option<&Holding>), With<Player>>,
    containers: Query<&ContainableItems>,
    targets: Query<(), With<Targetted>>,
    water_icon: Option<Single<Entity, With<WaterIcon>>>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    let (player, near_water, held_item) = *player;

    let can_hold = if let Some(holding) = held_item {
        // Check if the item the player is holding is a container that can hold water
        let item = holding.iter().next().unwrap();

        containers
            .get(item)
            .is_ok_and(|containables| containables.0.contains(&ItemType::Water))
    } else {
        // Not holding anything
        true
    };

    let show_icon = targets.is_empty() && near_water && can_hold;

    match (show_icon, water_icon) {
        (true, None) => {
            // Spawn icon as child to player
            let icon_entity = commands
                .spawn((
                    ChildOf(player),
                    WaterIcon,
                    // Render
                    Transform::from_xyz(-0.5, -0.5, Z_HELD_ITEM),
                ))
                .id();

            ItemSprite::Water.spawn_sprite(&mut commands, &sprite_sheets, Some(icon_entity));
        }
        (false, Some(entity)) => {
            // Despawn the icon
            commands.entity(*entity).despawn();
        }
        _ => (),
    }
}

/// Pick up some water from an infinite source
pub fn harvest_water(
    mut commands: Commands,
    player: Single<(Entity, Option<&Holding>), (With<Player>, With<NearWater>)>,
    containers: Query<&ContainableItems>,
    sprite_sheets: Res<SpriteSheets>,
) {
    let container = player.1.and_then(|holding| {
        let item = holding.iter().next().unwrap();

        containers
            .get(item)
            .ok()
            .filter(|containables| containables.0.contains(&ItemType::Water))
            .map(|_| item)
    });

    // Spawn water item in the void
    let item = commands.spawn(ItemType::Water).id();
    ItemSprite::Water.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

    if let Some(container) = container {
        // Put the item in the container
        commands
            .entity(item)
            .insert(ContainedBundle::new(container));
    } else {
        // Give item to player directly
        commands.entity(item).insert(HeldItemBundle::new(player.0));
    }

    commands.trigger(HarvestEvent {
        resource_node: ResourceNodeType::Water,
        amount: RESOURCE_PICKUP_AMOUNT,
    });
}
