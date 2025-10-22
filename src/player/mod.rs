mod components;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

use crate::utils::run_if::{empty_hands, key_just_pressed};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    move_player,
                    target_thing,
                    harvest_resource.run_if(key_just_pressed(KeyCode::Space).and(empty_hands)),
                    check_near_water,
                    show_water_icon,
                    harvest_water.run_if(
                        key_just_pressed(KeyCode::Space)
                            .and(empty_hands)
                            // Targets take precedence
                            .and(|targets: Query<(), With<Targetted>>| targets.is_empty()),
                    ),
                ),
            )
            .add_observer(highlight_target)
            .add_observer(unhighlight_target)
            .add_observer(make_untargettable);
    }
}
