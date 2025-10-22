mod components;
mod systems;

use bevy::prelude::*;

pub use components::*;
use systems::*;

use crate::utils::run_if::key_just_pressed;

pub struct ContainerPlugin;
impl Plugin for ContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, contain_item.run_if(key_just_pressed(KeyCode::KeyC)));
    }
}
