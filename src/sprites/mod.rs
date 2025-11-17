mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        // Resource loading before any game stuff
        app.add_systems(PreStartup, load_sprite_sheets)
            .add_systems(Update, tilemap_post_load);
    }
}
