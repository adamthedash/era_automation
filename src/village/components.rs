use bevy::prelude::*;
use std::collections::HashMap;

use crate::resources::ResourceType;

/// Tracks the amount of a resource stored in the village
#[derive(Component)]
pub struct ResourceStockpile(pub f32);

/// Resource drain rate per second
#[derive(Component)]
pub struct ResourceDrainRate(pub f32);

/// Display name for resource
#[derive(Component)]
pub struct ResourceName(pub String);

/// Lookup table for stockpile entities
#[derive(Resource, Default)]
pub struct StockpileLut(pub HashMap<ResourceType, Entity>);

/// Marker for village building
#[derive(Component)]
pub struct VillageCentre;

/// Event triggered whenever the player deposits a resource into the village
#[derive(Debug, Event)]
pub struct DepositEvent {
    pub resource: ResourceType,
    pub amount: usize,
}
