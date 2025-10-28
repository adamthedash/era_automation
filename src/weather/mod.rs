mod components;
pub mod systems;
use std::f32::consts::PI;

pub use components::*;
use rand::random_range;
use systems::*;

use bevy::prelude::*;

pub struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (spawn_fluff, tick_fluffs))
            .insert_resource(Wind {
                direction: Vec2::from_angle(random_range(0.0..(2. * PI))),
                speed: 1.,
            });
    }
}
