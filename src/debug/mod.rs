mod components;
mod systems;

use bevy::prelude::*;
use components::*;
use systems::*;

use crate::utils::run_if::key_just_pressed;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GradientArrowLUT>()
            .init_resource::<GradientArrowsEnabled>()
            .add_systems(
                FixedUpdate,
                (
                    toggle_gradient_arrows.run_if(key_just_pressed(KeyCode::KeyG)),
                    (spawn_gradient_arrows, update_gradient_arrows)
                        .chain()
                        .run_if(resource_equals(GradientArrowsEnabled(true))),
                    despawn_gradient_arrows.run_if(resource_equals(GradientArrowsEnabled(false))),
                ),
            );
    }
}
