use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    consts::Z_RESOURCES,
    ground_items::GroundItemBundle,
    items::ItemType,
    map::{TilePos, WorldPos},
    player::Player,
    resources::{ResourceAmount, ResourceMarker, ResourceNodeLUT, ResourceNodeType},
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

use super::components::*;

/// Advance the state of the harvesters if there is a resource beside it
pub fn tick_harvesters(
    machines: Query<
        (
            &TilePos,
            &mut HarvestState,
            &HarvestSpeed,
            &Direction,
            &HarvestableNodes,
        ),
        With<Harvester>,
    >,
    resource_lut: Res<ResourceNodeLUT>,
    resources: Query<(&ResourceNodeType, &ItemType), With<ResourceMarker>>,
    timer: Res<Time>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (tile_pos, mut state, speed, direction, harvestable_nodes) in machines {
        // Check if there's a harvestable node in front of the machine
        let resource_pos = TilePos(tile_pos.0 + direction.0);

        let Some(resource) = resource_lut.0.get(&resource_pos) else {
            // No resource, so reset progress
            state.0 = 0.;
            continue;
        };

        // Check that resource can be harvested by this machine
        let (resource_type, item_type) = resources.get(*resource).expect("Resource node not found");
        if !harvestable_nodes.0.contains(resource_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Tick the machine
        state.0 += timer.delta_secs();

        // Check if harvest has been completed
        if state.0 >= speed.0 {
            state.0 -= speed.0;

            let behind = TilePos(tile_pos.0 - direction.0);

            // Spawn an item on the ground
            let entity = commands
                .spawn((
                    GroundItemBundle::new(&behind.as_world_pos()),
                    // Game data
                    *item_type,
                    ResourceAmount(1),
                ))
                .id();

            item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(entity));
        }
    }
}

/// Place a harvester at the player's feet
pub fn spawn_harvester(
    player: Single<&WorldPos, With<Player>>,
    mut machines: ResMut<MachineLUT>,
    resources: Res<ResourceNodeLUT>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    let tile_pos = player.tile();

    if machines.0.contains_key(&tile_pos) {
        // Machine already here
        return;
    }
    if resources.0.contains_key(&tile_pos) {
        // Resource already here
        return;
    }

    // Spawn the machine
    let machine = commands
        .spawn((
            tile_pos,
            tile_pos.as_transform(Z_RESOURCES),
            // TODO: Harvester bundle
            Machine,
            Harvester,
            Direction(-IVec2::X),
            HarvestSpeed(2.),
            HarvestState(0.),
            HarvestableNodes({
                let mut hs = HashSet::new();
                hs.insert(ResourceNodeType::Bush);
                hs
            }),
        ))
        .id();

    ResourceSprite::House.spawn_sprite(&mut commands, &sprite_sheets, Some(machine));

    machines.0.insert(tile_pos, machine);
}
