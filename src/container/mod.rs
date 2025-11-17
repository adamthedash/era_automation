mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

use crate::{player::TargettedBy, utils::run_if::key_just_pressed};

pub struct ContainerPlugin;
impl Plugin for ContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                contain_item,
                uncontain_item
                    // Target takes precedence
                    .run_if(|targets: Query<(), With<TargettedBy>>| targets.is_empty()),
            )
                .run_if(key_just_pressed(KeyCode::KeyC)),
        );
    }
}
