mod components;
mod systems;

use bevy::prelude::*;

pub use components::*;
use systems::*;

use crate::{player::Targetted, utils::run_if::key_just_pressed};

/// Controls village resources which need to be sustained
pub struct VillagePlugin;
impl Plugin for VillagePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<StockpileLut>()
            .add_systems(Startup, (setup_stockpiles, setup_resource_display).chain())
            .add_systems(Startup, spawn_village_centre)
            .add_systems(
                Update,
                (
                    update_resources,
                    update_resource_display,
                    deposit_resource.run_if(key_just_pressed(KeyCode::Space).and(
                        // Only run when targetting a village centre
                        |village: Query<(), (With<VillageCentre>, With<Targetted>)>| {
                            !village.is_empty()
                        },
                    )),
                ),
            );
    }
}
