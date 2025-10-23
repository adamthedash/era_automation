mod components;
pub mod systems;

use bevy::prelude::*;

pub use components::*;
use systems::*;

/// Controls world generation
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_world_gen)
            .add_systems(
                Update,
                (create_chunks, update_transforms, update_tilemap_data),
            )
            .add_systems(FixedUpdate, recompute_gradients)
            .add_observer(update_gradient_map)
            .init_resource::<ChunkLUT>()
            .add_message::<CreateChunk>()
            .add_message::<RecomputeGradient>();
    }
}
