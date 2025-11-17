use bevy::prelude::*;

use super::components::*;
use crate::{
    ground_items::{GroundItem, GroundItemBundle},
    items::ItemType,
    map::WorldPos,
    player::{HeldBy, Player, TargettedBy},
};

/// Pick up a ground item and put it into the container
pub fn contain_item(
    item: Single<(Entity, &ItemType), (With<GroundItem>, With<TargettedBy>)>,
    container: Single<(Entity, &ContainableItems), (With<Container>, With<HeldBy>)>,
    mut commands: Commands,
) {
    if !container.1.0.contains(item.1) {
        // Item can't be put in this container
        return;
    }

    info!("Containing item {:?}", item.0);

    commands
        .entity(item.0)
        // Remove ground related components
        .remove::<GroundItemBundle>()
        // Add containing related components
        .insert(ContainedBundle::new(container.0));
}

/// Take an item out of the held container and put it on the ground
pub fn uncontain_item(
    container: Single<(Entity, &Contains), (With<Container>, With<HeldBy>)>,
    player_pos: Single<&WorldPos, With<Player>>,
    mut commands: Commands,
) {
    let item = container
        .1
        .iter()
        .next()
        .expect("System only runs when there are contained items");

    info!("Un-containing item {:?}", item);

    commands
        .entity(item)
        .remove::<ContainedBundle>()
        .insert(GroundItemBundle::new(*player_pos));
}
