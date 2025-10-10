use bevy::{
    self,
    ecs::{bundle::InsertMode, system::entity_command},
    prelude::*,
};

use crate::{
    consts::{PLAYER_REACH, PLAYER_SPEED, TILE_DISPLAY_SIZE, TILE_RAW_SIZE, Z_PLAYER},
    map::{TilePos, WorldPos},
    resources::{ResourceMarker, ResourceType},
    sprites::{EntitySprite, SpriteSheets},
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, (move_player, target_resource, pickup_resource))
            .add_observer(highlight_target)
            .add_observer(unhighlight_target);
    }
}

#[derive(Component)]
pub struct Player;
fn setup_player(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
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
        // Z == 2 for player
        world_pos.as_transform(Z_PLAYER),
        Player,
    ));
}

fn move_player(
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
/// Pick up a resource and put it in the player's hand
fn pickup_resource(
    mut commands: Commands,
    player: Single<(Entity, &Transform), With<Player>>,
    inputs: Res<ButtonInput<KeyCode>>,
    targetted_resources: Query<
        (Entity, &Sprite, &ResourceType),
        (With<ResourceMarker>, With<Targetted>),
    >,
    held_item: Option<Single<(), With<HeldItem>>>,
) {
    if inputs.pressed(KeyCode::Space) {
        if held_item.is_some() {
            // Already holding something
            return;
        }

        if let Some((resource_entity, sprite, res_type)) = targetted_resources.iter().next() {
            // Add item to player
            commands.entity(player.0).with_children(|parent| {
                parent.spawn((
                    // Z == 1 for held items
                    Transform::from_translation(
                        (Vec2::splat(0.5) * TILE_DISPLAY_SIZE.as_vec2()
                            // Need to un-scale so offset is ok
                            / player.1.scale.truncate())
                        .extend(1.),
                    ),
                    sprite.clone(),
                    *res_type,
                    HeldItem,
                ));
            });

            // Remove resource
            commands.entity(resource_entity).despawn();
        }
    }
}
