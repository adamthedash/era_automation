pub mod bundles;
mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

pub struct ResourcePlugin;
impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceNodeLUT>()
            .add_observer(spawn_resources)
            .add_systems(FixedUpdate, (regenerate_resource_nodes, mark_resource_full))
            .add_systems(Update, sync_resource_sprites);
    }
}
