use bevy::{
    self,
    ecs::{bundle::InsertMode, system::entity_command},
    prelude::*,
};

use crate::{
    consts::{
        PLAYER_REACH, PLAYER_SPEED, RESOURCE_PICKUP_AMOUNT, TILE_DISPLAY_SIZE, TILE_RAW_SIZE,
        Z_PLAYER,
    },
    map::{ChunkLUT, TerrainData, TilePos, WorldPos},
    resources::{ResourceAmount, ResourceMarker, ResourceType},
    sprites::{EntitySprite, ResourceSprite, SpriteSheets, TerrainSprite},
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    move_player,
                    target_resource,
                    pickup_resource,
                    check_near_water,
                    show_water_icon,
                    pickup_water,
                ),
            )
            .add_observer(highlight_target)
            .add_observer(unhighlight_target);
    }
}

#[derive(Component)]
pub struct Player;
pub fn setup_player(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
    let world_pos = WorldPos(Vec2::ZERO);
    commands.spawn((Camera2d, world_pos, Transform::IDENTITY));

    commands.spawn((
        Sprite::from_atlas_image(
            sprite_sheets.entities.image.clone(),
            TextureAtlas {
                layout: sprite_sheets.entities.layout.clone(),
                index: EntitySprite::Player as usize,
            },
        ),
        world_pos,
        world_pos.as_transform(Z_PLAYER),
        Player,
    ));
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

#[derive(Component)]
pub struct Targettable;
#[derive(Component)]
pub struct Targetted;
/// Targets the closest thing to the player
fn target_resource(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    targettables: Query<(Entity, &TilePos), With<Targettable>>,
    targetted: Query<Entity, (With<Targettable>, With<Targetted>)>,
) {
    let player_world_pos = WorldPos::from_transform(&player);

    let closest = targettables
        .iter()
        .map(|(entity, tile_pos)| {
            let distance2 = player_world_pos
                .0
                .distance_squared(tile_pos.as_world_pos().0);

            (entity, tile_pos, distance2)
        })
        .filter(|(_, _, distance2)| *distance2 <= PLAYER_REACH.powi(2))
        .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(d2));

    // TODO: Don't keep removing + adding if it's the same target
    for entity in targetted {
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

/// Make targetted resources bigger
fn highlight_target(
    event: On<Add, Targetted>,
    mut transforms: Query<&mut Transform, With<Targetted>>,
) {
    if let Ok(mut transform) = transforms.get_mut(event.entity) {
        transform.scale = (1.2 * TILE_DISPLAY_SIZE.as_vec2() / TILE_RAW_SIZE.as_vec2()).extend(1.);
    }
}
/// Make untargetted resources smaller
fn unhighlight_target(
    event: On<Remove, Targetted>,
    mut transforms: Query<&mut Transform, Added<Targetted>>,
) {
    if let Ok(mut transform) = transforms.get_mut(event.entity) {
        transform.scale = (1. * TILE_DISPLAY_SIZE.as_vec2() / TILE_RAW_SIZE.as_vec2()).extend(1.);
    }
}

#[derive(Component)]
pub struct HeldItem;

#[derive(Event, Debug)]
pub struct HarvestEvent {
    pub resource_type: ResourceType,
    pub amount: usize,
    // TODO: Node type / position?
}

/// Pick up a resource and put it in the player's hand
fn pickup_resource(
    mut commands: Commands,
    player: Single<(Entity, &Transform), With<Player>>,
    inputs: Res<ButtonInput<KeyCode>>,
    mut targetted_resources: Query<
        (Entity, &Sprite, &ResourceType, &mut ResourceAmount),
        (With<ResourceMarker>, With<Targetted>),
    >,
    held_item: Option<Single<(), With<HeldItem>>>,
) {
    if inputs.pressed(KeyCode::Space) {
        if held_item.is_some() {
            // Already holding something
            return;
        }

        if let Some((resource_entity, sprite, res_type, mut amount)) =
            targetted_resources.iter_mut().next()
        {
            let pickup_amount = RESOURCE_PICKUP_AMOUNT.min(amount.0);

            // Add item to player
            commands.entity(player.0).with_children(|parent| {
                parent.spawn((
                    // Game data
                    *res_type,
                    ResourceAmount(pickup_amount),
                    HeldItem,
                    // Render
                    Transform::from_translation(
                        (Vec2::splat(0.5) * TILE_DISPLAY_SIZE.as_vec2()
                            // Need to un-scale so offset is ok
                            / player.1.scale.truncate())
                        // Z == 1 for held items
                        .extend(1.),
                    ),
                    sprite.clone(),
                ));
            });

            // Remove resource if it's depleted
            if amount.0 == pickup_amount {
                // Player has grabbed it all, so remove the node
                commands.entity(resource_entity).despawn();
            } else {
                // Player has only picked up some of it
                amount.0 -= pickup_amount;
            }

            commands.trigger(HarvestEvent {
                resource_type: *res_type,
                amount: pickup_amount,
            });
        }
    }
}

#[derive(Component)]
struct NearWater;
/// Checks whether the player is in range of a water source
fn check_near_water(
    player: Single<(Entity, &WorldPos), With<Player>>,
    chunks_lut: Res<ChunkLUT>,
    tile_data: Query<&TerrainData>,
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
            let tile_data = chunks_lut
                .0
                .get(&chunk_pos)
                .and_then(|entity| tile_data.get(*entity).ok());

            tile_data.map(|td| (offset, td))
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

#[derive(Component)]
struct WaterIcon;
/// Shows the water icon when the player is near the water
fn show_water_icon(
    player: Single<(Entity, Option<&NearWater>, &Transform), With<Player>>,
    targets: Query<(), With<Targetted>>,
    water_icon: Option<Single<Entity, With<WaterIcon>>>,
    held_item: Query<(), With<HeldItem>>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    let (player, near_water, transform) = *player;

    let show_icon = targets.is_empty() && near_water.is_some() && held_item.is_empty();

    if show_icon {
        if water_icon.is_none() {
            // Spawn icon as child to player
            commands.entity(player).with_child((
                WaterIcon,
                // Render
                Transform::from_translation(
                    (Vec2::splat(-0.5) * TILE_DISPLAY_SIZE.as_vec2()
                // Need to un-scale so offset is ok
                / transform.scale.truncate())
                    // Z == 1 for held items
                    .extend(1.),
                ),
                Sprite {
                    image: sprite_sheets.resources.image.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: sprite_sheets.resources.layout.clone(),
                        index: ResourceSprite::Water as usize,
                    }),
                    ..Default::default()
                },
            ));
        }
    } else if let Some(entity) = water_icon {
        // Despawn the icon
        commands.entity(*entity).despawn();
    }
}

/// Pick up some water from an infinite source
fn pickup_water(
    mut commands: Commands,
    player: Single<(Entity, &Transform), (With<Player>, With<NearWater>)>,
    inputs: Res<ButtonInput<KeyCode>>,
    targets: Query<(), With<Targetted>>,
    held_item: Option<Single<(), With<HeldItem>>>,
    sprite_sheets: Res<SpriteSheets>,
) {
    if !inputs.pressed(KeyCode::Space) {
        return;
    }

    if held_item.is_some() {
        // Already holding something
        return;
    }

    if !targets.is_empty() {
        // Targets take precedence
        return;
    }

    // Add item to player
    commands.entity(player.0).with_children(|parent| {
        parent.spawn((
            // Game data
            ResourceType::Water,
            ResourceAmount(RESOURCE_PICKUP_AMOUNT),
            HeldItem,
            // Render
            Transform::from_translation(
                (Vec2::splat(0.5) * TILE_DISPLAY_SIZE.as_vec2()
                            // Need to un-scale so offset is ok
                            / player.1.scale.truncate())
                // Z == 1 for held items
                .extend(1.),
            ),
            Sprite {
                image: sprite_sheets.resources.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheets.resources.layout.clone(),
                    index: ResourceSprite::Water as usize,
                }),
                ..Default::default()
            },
        ));
    });

    commands.trigger(HarvestEvent {
        resource_type: ResourceType::Water,
        amount: RESOURCE_PICKUP_AMOUNT,
    });
}
