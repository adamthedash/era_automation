use bevy::prelude::*;

use crate::{
    crafting::Recipe, ground_items::ItemRolled, items::ItemType, player::HarvestEvent,
    resources::ResourceType, village::DepositEvent,
};

use super::components::*;

/// Initialise the knowledge
/// TODO: Move to data file
pub fn init_knowledge(mut commands: Commands) {
    commands.spawn((
        UnlockName("Bowl".to_string()),
        UnlockRequirements(vec![
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 5,
            },
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Water,
                amount: 5,
            },
        ]),
        Recipe {
            reqs: vec![(ResourceType::Wood, 5)],
            product: ItemType::Bowl,
        },
    ));
    commands.spawn((
        UnlockName("Harvester".to_string()),
        UnlockRequirements(vec![
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Wood,
                amount: 0,
            },
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Water,
                amount: 0,
            },
        ]),
        Recipe {
            reqs: vec![(ResourceType::Wood, 5)],
            product: ItemType::Harvester,
        },
    ));
    commands.spawn((
        UnlockName("Transporter".to_string()),
        UnlockRequirements(vec![UnlockRequirement::TotalRolled {
            item: ItemType::Log,
            distance: 10.,
        }]),
        Recipe {
            reqs: vec![(ResourceType::Wood, 5)],
            product: ItemType::Transporter,
        },
    ));
    commands.spawn((
        UnlockName("Plant Watering".to_string()),
        UnlockRequirements(vec![
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Food,
                amount: 2,
            },
            UnlockRequirement::TotalDeposited {
                resource: ResourceType::Water,
                amount: 2,
            },
        ]),
    ));
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
