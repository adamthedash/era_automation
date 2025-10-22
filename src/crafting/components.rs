use bevy::prelude::*;

use crate::{items::ItemType, resources::ResourceType};

/// Marker for when the player is nearby a crafting station
#[derive(Component)]
pub struct NearCraftingStation;

/// Resource requirements to craft an item
#[derive(Component, Clone, Debug)]
pub struct Recipe {
    pub reqs: Vec<(ResourceType, usize)>,
    pub product: ItemType,
}

/// Top-level marker for crafting UI
#[derive(Component)]
pub struct CraftingWindow;

/// Marker for recipe-level UI
#[derive(Component)]
pub struct CraftingNode;

/// Message instructing a recipe to be crafted
#[derive(Message)]
pub struct CraftRecipe(pub Recipe);

/// Error types when a craft is attempted but fails
#[derive(Debug)]
pub enum FailedCraftReason {
    NotEnoughResources,
    HoldingItem,
}

/// Event thrown when a craft is attempted but fails
#[derive(Event)]
pub struct FailedCraft {
    pub recipe: Recipe,
    pub reason: FailedCraftReason,
}
