mod components;
pub mod systems;

pub use components::*;
use systems::*;

use bevy::prelude::*;

pub struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (spawn_fluff, tick_fluffs, change_wind_direction),
        )
        .init_resource::<Wind>();
    }
}
