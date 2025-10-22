use bevy::{platform::collections::HashSet, prelude::*};

use crate::{consts::Z_CONTAINED_ITEM, items::ItemType};

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

#[derive(Bundle)]
pub struct ContainedBundle {
    parent: ChildOf,
    container: ContainedBy,
    transform: Transform,
}
impl ContainedBundle {
    pub fn new(container: Entity) -> Self {
        Self {
            parent: ChildOf(container),
            container: ContainedBy(container),
            // Render contained item above/behind container
            transform: Transform::from_xyz(0., 0.5, Z_CONTAINED_ITEM),
        }
    }
}
