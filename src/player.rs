use bevy::{
    self,
    ecs::{bundle::InsertMode, system::entity_command},
    prelude::*,
};

use crate::{
    consts::{
        PLAYER_REACH, PLAYER_SPEED, RESOURCE_PICKUP_AMOUNT, TILE_DISPLAY_SIZE, TILE_RAW_SIZE,
        Z_HELD_ITEM, Z_PLAYER,
    },
    map::{ChunkLUT, TerrainData, TilePos, WorldPos},
    resources::{ResourceAmount, ResourceMarker, ResourceType},
    sprites::{EntitySprite, ResourceSprite, SpriteSheet, SpriteSheets, TerrainSprite},
    utils::run_if::key_just_pressed,
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    move_player,
                    target_thing,
                    pickup_resource.run_if(key_just_pressed(KeyCode::Space)),
                    check_near_water,
                    show_water_icon,
                    pickup_water.run_if(key_just_pressed(KeyCode::Space)),
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
        Player,
        world_pos,
        // Render
        world_pos.as_transform(Z_PLAYER),
        Sprite::from_atlas_image(
            sprite_sheets.entities.image.clone(),
            TextureAtlas {
                layout: sprite_sheets.entities.layout.clone(),
                index: EntitySprite::Player as usize,
            },
        ),
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
fn target_thing(
    mut commands: Commands,
    player: Single<&WorldPos, With<Player>>,
    targettables: Query<(Entity, Option<&TilePos>, Option<&WorldPos>), With<Targettable>>,
    targetted: Query<Entity, (With<Targettable>, With<Targetted>)>,
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
    mut targetted_resources: Query<
        (Entity, &ResourceType, &mut ResourceAmount),
        (With<ResourceMarker>, With<Targetted>),
    >,
    held_item: Option<Single<(), With<HeldItem>>>,
    sprite_sheets: Res<SpriteSheets>,
) {
    if held_item.is_some() {
        // Already holding something
        return;
    }

    if let Some((resource_entity, res_type, mut amount)) = targetted_resources.iter_mut().next() {
        let pickup_amount = RESOURCE_PICKUP_AMOUNT.min(amount.0);

        // Add item to player
        commands.entity(player.0).with_children(|parent| {
            parent.spawn((
                // Game data
                *res_type,
                ResourceAmount(pickup_amount),
                held_item_bundle(res_type.sprite(), &sprite_sheets.resources, player.1),
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

    match (show_icon, water_icon) {
        (true, None) => {
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
        (false, Some(entity)) => {
            // Despawn the icon
            commands.entity(*entity).despawn();
        }
        _ => (),
    }
}

/// Bundle of components for spawning a held item for the the player
pub fn held_item_bundle(
    sprite: impl Into<usize>,
    sprite_sheet: &SpriteSheet,
    player_transform: &Transform,
) -> impl Bundle {
    (
        HeldItem,
        // Render
        Transform::from_translation(
            // Need to un-scale so offset is ok
            (Vec2::splat(0.5) * TILE_DISPLAY_SIZE.as_vec2() / player_transform.scale.truncate())
                .extend(Z_HELD_ITEM),
        ),
        Sprite {
            image: sprite_sheet.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheet.layout.clone(),
                index: sprite.into(),
            }),
            ..Default::default()
        },
    )
}

/// Pick up some water from an infinite source
fn pickup_water(
    mut commands: Commands,
    player: Single<(Entity, &Transform), (With<Player>, With<NearWater>)>,
    targets: Query<(), With<Targetted>>,
    held_item: Option<Single<(), With<HeldItem>>>,
    sprite_sheets: Res<SpriteSheets>,
) {
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
            held_item_bundle(ResourceSprite::Water, &sprite_sheets.resources, player.1),
        ));
    });

    commands.trigger(HarvestEvent {
        resource_type: ResourceType::Water,
        amount: RESOURCE_PICKUP_AMOUNT,
    });
}
