use bevy::{platform::collections::HashMap, prelude::*};

use crate::{crafting::Recipe, player::HarvestEvent, resources::ResourceType};

pub struct KnowledgePlugin;
impl Plugin for KnowledgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringStatistics>()
            .add_systems(Startup, init_knowledge)
            .add_systems(Update, check_unlocks)
            .add_observer(update_statistics);
    }
}

#[derive(Component)]
pub struct Unlocked;

#[derive(Component, Debug)]
pub struct UnlockName(pub String);

enum UnlockRequirement {
    TotalGathered {
        resource: ResourceType,
        amount: usize,
    },
}
#[derive(Component)]
struct UnlockRequirements(Vec<UnlockRequirement>);

/// Initialise the knowledge
/// TODO: Move to data file
fn init_knowledge(mut commands: Commands) {
    commands.spawn((
        UnlockName("Bowl".to_string()),
        UnlockRequirements(vec![
            UnlockRequirement::TotalGathered {
                resource: ResourceType::Wood,
                amount: 1,
            },
            UnlockRequirement::TotalGathered {
                resource: ResourceType::Water,
                amount: 0,
            },
        ]),
        Recipe {
            reqs: vec![(ResourceType::Wood, 5)],
            product: ResourceType::Bowl,
        },
    ));
    commands.spawn((
        UnlockName("Plant Watering".to_string()),
        UnlockRequirements(vec![
            UnlockRequirement::TotalGathered {
                resource: ResourceType::Food,
                amount: 2,
            },
            UnlockRequirement::TotalGathered {
                resource: ResourceType::Water,
                amount: 2,
            },
        ]),
    ));
}

#[derive(Event)]
pub struct UnlockEvent {
    pub name: String,
}

/// Checks all of the knowledge and unlocks ones that have met their requirements
fn check_unlocks(
    query: Query<(Entity, &UnlockRequirements, &UnlockName), Without<Unlocked>>,
    stats: Res<GatheringStatistics>,
    mut commands: Commands,
) {
    for (entity, requirements, name) in query {
        // Check that all requirements are met
        if requirements.0.iter().all(|req| match req {
            UnlockRequirement::TotalGathered { resource, amount } => {
                stats.total_gathered.get(resource).unwrap_or(&0) >= amount
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

/// Tracks lifetime statistics for the player
#[derive(Resource, Default)]
struct GatheringStatistics {
    total_gathered: HashMap<ResourceType, usize>,
}
/// Update lifetime statistics
fn update_statistics(event: On<HarvestEvent>, mut stats: ResMut<GatheringStatistics>) {
    info!("Updating stats: {:?}", event);
    *stats.total_gathered.entry(event.resource_type).or_default() += event.amount;
}
