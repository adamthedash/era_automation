use bevy::prelude::*;

use super::{components::*, data};
use crate::{ground_items::ItemRolled, player::HarvestEvent, village::DepositEvent};

/// Initialise the knowledge
/// Knowledge definititions are stored in the data module
pub fn init_knowledge(mut commands: Commands) {
    for knowledge in data::load_knowledge() {
        // Spawn the base components (name + requirements)
        let mut entity = commands.spawn((
            UnlockName(knowledge.name),
            UnlockRequirements(knowledge.requirements),
        ));

        // If there is a recipe associated with the knowledge, insert it.
        if let Some(recipe) = knowledge.recipe {
            entity.insert(recipe);
        }
    }
}

/// Debug system to unlock everything
pub fn unlock_everything(
    mut commands: Commands,
    unlockables: Query<(Entity, &UnlockName), Without<Unlocked>>,
) {
    for (entity, name) in unlockables {
        // Add the Unlocked tag
        info!("Unlocked knowledge: {:?}", name);
        commands.entity(entity).insert(Unlocked);
        commands.trigger(UnlockEvent {
            name: name.0.clone(),
        });
    }
}

/// Checks all of the knowledge and unlocks ones that have met their requirements
pub fn check_unlocks(
    query: Query<(Entity, &UnlockRequirements, &UnlockName), Without<Unlocked>>,
    stats: Res<GatheringStatistics>,
    mut commands: Commands,
) {
    for (entity, requirements, name) in query {
        // Check that all requirements are met
        if requirements.0.iter().all(|req| match req {
            UnlockRequirement::TotalGathered { resource, amount } => {
                stats.nodes_gathered.get(resource).unwrap_or(&0) >= amount
            }
            UnlockRequirement::TotalDeposited { resource, amount } => {
                stats.resources_deposited.get(resource).unwrap_or(&0) >= amount
            }
            UnlockRequirement::TotalRolled { item, distance } => {
                stats.items_rolled.get(item).unwrap_or(&0.) >= distance
            }
        }) {
            // Add the Unlocked tag
            info!("Unlocked knowledge: {:?}", name);
            commands.entity(entity).insert(Unlocked);
            commands.trigger(UnlockEvent {
                name: name.0.clone(),
            });
        }
    }
}

/// Update lifetime statistics
pub fn update_harvest_statistics(event: On<HarvestEvent>, mut stats: ResMut<GatheringStatistics>) {
    info!("Updating stats: {:?}", event);
    *stats.nodes_gathered.entry(event.resource_node).or_default() += event.amount;
}

/// Update lifetime statistics
pub fn update_deposit_statistics(event: On<DepositEvent>, mut stats: ResMut<GatheringStatistics>) {
    info!("Updating stats: {:?}", event);
    *stats.resources_deposited.entry(event.resource).or_default() += event.amount;
}

/// Update lifetime statistics
pub fn update_roll_statistics(event: On<ItemRolled>, mut stats: ResMut<GatheringStatistics>) {
    // info!("Updating stats: {:?}", event);
    *stats.items_rolled.entry(event.item).or_default() += event.distance;
}
