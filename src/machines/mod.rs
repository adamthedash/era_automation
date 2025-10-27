mod components;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

use crate::utils::run_if::key_just_pressed;

pub struct MachinePlugin;
impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MachineLUT>()
            .add_systems(FixedUpdate, (tick_harvesters, tick_transporters))
            .add_systems(
                Update,
                (
                    place_machine.run_if(key_just_pressed(KeyCode::KeyP)),
                    animate_machine,
                ),
            );
    }
}
