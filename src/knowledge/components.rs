use bevy::{platform::collections::HashMap, prelude::*};

use crate::resources::{ResourceNodeType, ResourceType};

/// Marker trait for knowledge which has been unlocked
#[derive(Component)]
pub struct Unlocked;

/// Display name for knowledge
#[derive(Component, Debug)]
pub struct UnlockName(pub String);

pub enum UnlockRequirement {
    TotalGathered {
        resource: ResourceNodeType,
        amount: usize,
    },
    TotalDeposited {
        resource: ResourceType,
        amount: usize,
    },
}

/// List of requirements needed to unlock knowledge
#[derive(Component)]
pub struct UnlockRequirements(pub Vec<UnlockRequirement>);

/// Event triggered when some knowledge is unlocked
#[derive(Event)]
pub struct UnlockEvent {
    pub name: String,
}

/// Tracks lifetime statistics for the player
#[derive(Resource, Default)]
pub struct GatheringStatistics {
    pub nodes_gathered: HashMap<ResourceNodeType, usize>,
    pub resources_deposited: HashMap<ResourceType, usize>,
}
