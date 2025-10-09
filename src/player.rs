use bevy::prelude::*;

use crate::consts::{CAMERA_SPEED, TILE_DISPLAY_SIZE};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, move_player);
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
