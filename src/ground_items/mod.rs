mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

use crate::utils::run_if::{empty_hands, key_just_pressed};

pub struct GroundItemPlugin;
impl Plugin for GroundItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_items,
                (drop_item, pickup_item.run_if(empty_hands))
                    .run_if(key_just_pressed(KeyCode::KeyE)),
                roll_items,
            ),
        );
    }
}
