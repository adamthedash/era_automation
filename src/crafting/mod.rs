mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;
use systems::*;

pub struct CraftingPlugin;
impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CraftRecipe>().add_systems(
            Update,
            (
                check_near_crafting_station,
                show_crafting_ui,
                crafting_button,
                try_craft_recipes,
            ),
        );
    }
}
