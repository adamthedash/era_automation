mod components;
mod data;
mod systems;

use bevy::prelude::*;

pub use components::*;
use systems::*;

pub struct KnowledgePlugin;
impl Plugin for KnowledgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringStatistics>()
            .add_systems(Startup, (init_knowledge, unlock_everything).chain())
            .add_systems(Update, check_unlocks)
            .add_observer(update_harvest_statistics)
            .add_observer(update_deposit_statistics)
            .add_observer(update_roll_statistics);
    }
}
