mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

pub struct ResourcePlugin;
impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceNodeLUT>()
            .add_observer(spawn_resources);
    }
}
