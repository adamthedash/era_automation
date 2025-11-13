pub mod bundles;
mod components;
pub mod systems;

use bevy::prelude::*;
pub use bundles::*;
pub use components::*;
use systems::*;

use crate::utils::run_if::{empty_hands, key_just_pressed};

pub struct MachinePlugin;
impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MachineLUT>()
            .init_resource::<EnergyNetworks>()
            .add_message::<TransferItem>()
            .add_systems(
                FixedUpdate,
                (
                    compute_energy_networks,
                    (tick_windmills,),
                    produce_energy,
                    |mut networks: ResMut<EnergyNetworks>| {
                        networks.power_demands.clear();
                    },
                    (precheck_resource_harvesters,),
                    distribute_energy,
                    (
                        tick_resource_harvesters,
                        tick_terrain_harvesters,
                        tick_transporters,
                        tick_pickerupper,
                    ),
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
