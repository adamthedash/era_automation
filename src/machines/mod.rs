mod components;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

use crate::utils::run_if::{empty_hands, key_just_pressed};

pub struct MachinePlugin;
impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MachineLUT>()
            .add_message::<TransferItem>()
            .add_systems(
                FixedUpdate,
                (
                    (tick_harvesters, tick_transporters, tick_pickerupper),
                    transfer_items,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    (place_machine, pickup_machine.run_if(empty_hands))
                        .run_if(key_just_pressed(KeyCode::KeyP)),
                    rotate_machine.run_if(key_just_pressed(KeyCode::KeyR)),
                    animate_machine,
                ),
            );
    }
}
