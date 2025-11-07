use bevy::prelude::*;

use crate::{consts::Z_HELD_ITEM, resources::ResourceNodeType};

/// Marker for player
#[derive(Component)]
pub struct Player;

/// Marker for things that can be targetted / interacted with by the player
#[derive(Component)]
pub struct Targettable;

/// Relationship for when an entity is being targeted by something
#[derive(Component)]
#[relationship(relationship_target = Targetting)]
pub struct TargettedBy(pub Entity);

/// Relationship for when an entity is targeting one or more things
#[derive(Component, Debug)]
#[relationship_target(relationship = TargettedBy)]
pub struct Targetting(Vec<Entity>);

/// Relationship for when something is being held
#[derive(Component)]
#[relationship(relationship_target = Holding)]
pub struct HeldBy(pub Entity);

/// Relationship for when the entity is holding one or more things
#[derive(Component, Debug)]
#[relationship_target(relationship = HeldBy)]
pub struct Holding(Vec<Entity>);

/// Event thrown then a resource node is harvested by the player
#[derive(Event, Debug)]
pub struct HarvestEvent {
    pub resource_node: ResourceNodeType,
    pub amount: usize,
    // TODO: Node type / position?
}

/// Marker for when the player is nearby a water source
#[derive(Component)]
pub struct NearWater;

/// Marker for the harvest water action icon
#[derive(Component)]
pub struct WaterIcon;

/// Bundle of components for spawning a held item for the the player
#[derive(Bundle)]
pub struct HeldItemBundle {
    parent: ChildOf,
    holder: HeldBy,
    transform: Transform,
}
impl HeldItemBundle {
    pub fn new(holder: Entity) -> Self {
        Self {
            parent: ChildOf(holder),
            holder: HeldBy(holder),
            // Render off to the side of the player
            transform: Transform::from_xyz(0.5, 0.5, Z_HELD_ITEM),
        }
    }
}
