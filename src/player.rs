use bevy::{
    self,
    ecs::{bundle::InsertMode, system::entity_command},
    prelude::*,
};

use crate::{
    consts::{CAMERA_SPEED, PLAYER_REACH, TILE_DISPLAY_SIZE, TILE_RAW_SIZE},
    map::WorldPos,
    resources::{ResourceMarker, ResourceType},
    sprites::ResourceSprite,
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
fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Sprite::from_color(Color::WHITE, TILE_DISPLAY_SIZE.as_vec2()),
        Player,
        // Z == 2 for player
        Transform::from_xyz(0., 0., 2.),
    ));
}

fn move_player(
    mut player: Single<&mut Transform, With<Player>>,
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

    player.translation += (x * Vec3::X + y * Vec3::Y) * CAMERA_SPEED * timer.delta_secs();
}

#[derive(Component)]
pub struct Targettable;
#[derive(Component)]
pub struct Targetted;
/// Targets the closest thing to the player
fn target_resource(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    targettables: Query<(Entity, &Transform), With<Targettable>>,
    targetted: Query<(Entity, &Transform), (With<Targettable>, With<Targetted>)>,
) {
    let player_world_pos = WorldPos::from_transform(&player);

    let closest = targettables
        .iter()
        .map(|(entity, transform)| {
            let world_pos = WorldPos::from_transform(transform);
            let distance2 = player_world_pos.0.distance_squared(world_pos.0);

            (entity, transform, distance2)
        })
        .filter(|(_, _, distance2)| *distance2 <= PLAYER_REACH.powi(2))
        .min_by(|(_, _, d1), (_, _, d2)| d1.total_cmp(d2));

    // TODO: Don't keep removing + adding if it's the same target
    for (entity, _) in targetted {
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
    player: Single<Entity, With<Player>>,
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
            commands.entity(*player).with_children(|parent| {
                parent.spawn((
                    // Z == 1 for held items
                    Transform::from_translation(
                        (Vec2::splat(0.5) * TILE_DISPLAY_SIZE.as_vec2()).extend(1.),
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
