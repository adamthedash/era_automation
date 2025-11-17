use bevy::prelude::*;

use super::super::components::*;
use crate::{
    consts::RESOURCE_PICKUP_AMOUNT,
    ground_items::GroundItemBundle,
    items::ItemType,
    map::{Chunks, TerrainData, TilePos},
    resources::{ResourceAmount, ResourceMarker, ResourceNodeType, ResourceNodes},
    sprites::{GetSprite, SpriteSheets},
};

pub fn precheck_resource_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &PowerConsumption,
            &Direction,
            &HarvestableNodes,
        ),
        With<Harvester>,
    >,
    resources: ResourceNodes<(&ResourceNodeType, &ResourceAmount), With<ResourceMarker>>,
    mut energy_networks: ResMut<EnergyNetworks>,
) {
    for (tile_pos, mut state, power, direction, harvestable_nodes) in harvesters {
        let resource_pos = tile_pos + direction.0;

        // Check if there's a harvestable node in front of the machine
        let Some((resource_type, resource_amount)) = resources.get(&resource_pos) else {
            // No resource, so reset progress
            state.0 = 0.;
            continue;
        };

        // Check that there's some left to harvest
        if resource_amount.0 == 0 {
            // Resource is depleted, so reset progress
            state.0 = 0.;
            continue;
        }

        // Check that resource can be harvested by this machine
        if !harvestable_nodes.0.contains(resource_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Register power demand for this machine
        energy_networks.power_demands.insert(*tile_pos, power.0);
    }
}

/// Advance the state of the harvesters if there is a resource beside it
pub fn tick_resource_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &HarvestableNodes,
        ),
        With<Harvester>,
    >,
    mut resources: ResourceNodes<
        (&ResourceNodeType, &ItemType, &mut ResourceAmount),
        With<ResourceMarker>,
    >,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_networks: Res<EnergyNetworks>,
    timer: Res<Time>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (tile_pos, mut state, speed, power, direction, harvestable_nodes) in harvesters {
        // TODO: These pre-checks have already been done for power calculations

        // Check if there's a harvestable node in front of the machine
        let resource_pos = tile_pos + direction.0;
        let Some((resource_type, item_type, mut amount)) = resources.get_mut(&resource_pos) else {
            // No resource, so reset progress
            state.0 = 0.;
            continue;
        };

        // Check that the node has some resource left in it
        if amount.0 == 0 {
            // Resource is depleted, so reset progress
            state.0 = 0.;
            continue;
        }

        // Check that resource can be harvested by this machine
        if !harvestable_nodes.0.contains(resource_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        let energy_supply = energy_networks
            .power_provided
            .get(tile_pos)
            .expect("No power provided for this machine!");

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.);
        let work_rate = speed.0 * satisfaction;

        // Advance progress (1.0 == one completed action)
        state.0 += work_rate * timer.delta_secs();

        // Check if harvest has been completed
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Spawn an item
        let output_pos = tile_pos - direction.0;
        let item = commands.spawn(*item_type).id();
        item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&output_pos)
            && acceptable_items.can_accept(item_type)
        {
            info!("Transferring item Harvester -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Harvester -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&output_pos.as_world_pos()));
        }

        // Decrement resource amount now that we've produced an item.
        // TODO: Spawn same number of items that were picked up
        let pickup_amount = RESOURCE_PICKUP_AMOUNT.min(amount.0);
        amount.0 -= pickup_amount;
    }
}

pub fn precheck_terrain_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &PowerConsumption,
            &Direction,
            &HarvestableTerrain,
        ),
        With<Harvester>,
    >,
    chunks: Chunks<&TerrainData>,
    mut energy_networks: ResMut<EnergyNetworks>,
) {
    for (tile_pos, mut state, power, direction, harvestable_terrain) in harvesters {
        // Check if there's a harvestable node under of the machine
        let resource_pos = tile_pos + direction.0;

        // Get terrain under the machine
        let (chunk_pos, offset) = resource_pos.to_chunk_offset();
        let terrain_data = chunks.get(&chunk_pos).expect("Chunk data not generated");
        let terrain_type = &terrain_data.0[offset.y as usize][offset.x as usize];

        // Check that resource can be harvested by this machine
        if !harvestable_terrain.0.contains(terrain_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Check that the terrain can produce something
        if terrain_type.item_type().is_none() {
            // No item given
            state.0 = 0.;
            continue;
        };

        // Register power demand for this machine
        energy_networks.power_demands.insert(*tile_pos, power.0);
    }
}

/// Advance the state of the terrain harvesters
pub fn tick_terrain_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &HarvestableTerrain,
        ),
        With<Harvester>,
    >,
    chunks: Chunks<&TerrainData>,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_networks: Res<EnergyNetworks>,
    timer: Res<Time>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (tile_pos, mut state, speed, power, direction, harvestable_terrain) in harvesters {
        // TODO: These pre-checks have already been done for power calculations

        // Check if there's a harvestable node under of the machine
        let resource_pos = tile_pos + direction.0;

        // Get terrain under the machine
        let (chunk_pos, offset) = resource_pos.to_chunk_offset();
        let terrain_data = chunks.get(&chunk_pos).expect("Chunk data not generated");
        let terrain_type = &terrain_data.0[offset.y as usize][offset.x as usize];

        // Check that resource can be harvested by this machine
        if !harvestable_terrain.0.contains(terrain_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Check that the terrain can produce something
        let Some(item_type) = terrain_type.item_type() else {
            // No item given
            state.0 = 0.;
            continue;
        };

        // Accumulate energy produced by adjacent machines (e.g., windmills)
        let energy_supply = energy_networks
            .power_provided
            .get(tile_pos)
            .expect("No power provided for this machine!");

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        // Tick the machine (scaled by available adjacent energy)
        state.0 += work_rate * timer.delta_secs();

        // Check if harvest has been completed
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Spawn an item
        let output_pos = tile_pos + direction.0;
        let item = commands.spawn(item_type).id();
        item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&output_pos)
            && acceptable_items.can_accept(&item_type)
        {
            info!("Transferring item Harvester -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Harvester -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&output_pos.as_world_pos()));
        }
    }
}
