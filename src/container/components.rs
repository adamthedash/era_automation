use bevy::{platform::collections::HashSet, prelude::*};

use crate::items::ItemType;

/// Relationship for an item which is inside another item
#[derive(Component)]
#[relationship(relationship_target = Contains)]
pub struct ContainedBy(pub Entity);

/// Relationship for an item which contains other items
#[derive(Component)]
#[relationship_target(relationship = ContainedBy)]
pub struct Contains(Vec<Entity>);

/// Marker for item that can contain other items
#[derive(Component)]
pub struct Container;

/// The types of items this container can hold
#[derive(Component)]
pub struct ContainableItems(pub HashSet<ItemType>);
