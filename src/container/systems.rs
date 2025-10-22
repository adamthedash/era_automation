use super::components::*;
use bevy::prelude::*;

use crate::{
    consts::Z_CONTAINED_ITEM,
    ground_items::{AnimationCycleTime, GroundItem},
    map::WorldPos,
    player::{HeldBy, Targettable, Targetted},
};

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
    item.insert((
        ContainedBy(*container),
        // Render contained item above/behind container
        Transform::from_xyz(0., 0.5, Z_CONTAINED_ITEM),
    ));
}

/// Take an item out of the held container and put it on the ground
pub fn uncontain_item(container: Single<Entity, (With<Container>, With<HeldBy>)>) {
    info!("Un-containing item");
}
