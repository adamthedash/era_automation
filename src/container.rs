use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    ground_items::{AnimationCycleTime, GroundItem},
    items::ItemType,
    map::WorldPos,
    player::{HeldBy, Targettable, Targetted},
    utils::run_if::key_just_pressed,
};

pub struct ContainerPlugin;
impl Plugin for ContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, contain_item.run_if(key_just_pressed(KeyCode::KeyC)));
    }
}

#[derive(Component)]
#[relationship(relationship_target = Contains)]
pub struct ContainedBy(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ContainedBy)]
pub struct Contains(Vec<Entity>);

/// Marker for item that can contain other items
#[derive(Component)]
pub struct Container;

/// The types of items this container can hold
#[derive(Component)]
pub struct ContainableItems(pub HashSet<ItemType>);

/// Pick up a ground item and put it into the container
pub fn contain_item(
    item: Single<Entity, (With<GroundItem>, With<Targetted>)>,
    container: Single<Entity, (With<Container>, With<HeldBy>)>,
    mut commands: Commands,
) {
    info!("Containing item");

    // Move entity from world to player
    let mut item = commands.entity(*item);
    item.insert(ChildOf(*container));

    // Remove ground related components
    item.remove::<(GroundItem, Targettable, AnimationCycleTime, WorldPos)>();

    // Add containing related components
    item.insert((ContainedBy(*container), Transform::from_xyz(0., 0., 1.)));
}

/// Take an item out of the held container and put it on the ground
pub fn uncontain_item(container: Single<Entity, (With<Container>, With<HeldBy>)>) {
    //
}
