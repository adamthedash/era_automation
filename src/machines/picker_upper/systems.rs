use bevy::{platform::collections::HashMap, prelude::*};

use super::super::components::*;
use crate::{
    ground_items::{GroundItem, GroundItemBundle},
    items::ItemType,
    map::{TilePos, WorldPos},
};

pub fn precheck_pickeruppers(
    picker_uppers: Query<(&TilePos, &mut MachineState, &PowerConsumption), With<PickerUpper>>,
    ground_items: Query<(Entity, &WorldPos, &ItemType), With<GroundItem>>,
    mut energy_networks: ResMut<EnergyNetworks>,
) {
    // Create LUT for ground items
    // TODO: replace this with some spatial query data structure stored as a resource
    let ground_items = ground_items.iter().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut hm, (entity, world_pos, item_type)| {
            // NOTE: +0.5 so we search centre of tile instead of origin corner
            hm.entry((world_pos + Vec2::splat(0.5)).tile())
                .or_default()
                .push((entity, item_type));

            hm
        },
    );

    for (machine_pos, mut state, power) in picker_uppers {
        if !ground_items.contains_key(machine_pos) {
            // No items, reset progress
            state.0 = 0.;
            continue;
        }

        energy_networks.power_demands.insert(*machine_pos, power.0);
    }
}

/// Advance the state of the picker-upper if there's an item on its tile
pub fn tick_pickeruppers(
    picker_uppers: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
        ),
        With<PickerUpper>,
    >,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    ground_items: Query<(Entity, &WorldPos, &ItemType), With<GroundItem>>,
    energy_networks: Res<EnergyNetworks>,
    timer: Res<Time>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    // Create LUT for ground items
    // TODO: replace this with some spatial query data structure stored as a resource
    let ground_items = ground_items.iter().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut hm, (entity, world_pos, item_type)| {
            // NOTE: +0.5 so we search centre of tile instead of origin corner
            hm.entry((world_pos + Vec2::splat(0.5)).tile())
                .or_default()
                .push((entity, item_type));

            hm
        },
    );

    for (machine_pos, mut state, speed, power, direction) in picker_uppers {
        let Some(items) = ground_items.get(machine_pos) else {
            // No items, reset progress
            state.0 = 0.;
            continue;
        };

        // Accumulate energy produced by adjacent machines
        let energy_supply = energy_networks
            .power_provided
            .get(machine_pos)
            .expect("No power provided for this machine!");

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        // Advance state
        state.0 += work_rate * timer.delta_secs();
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Pick up an item
        let (item, item_type) = *items
            .first()
            .expect("If hashmap has entry, there should be at least 1 item");
        commands.entity(item).remove::<GroundItemBundle>();

        let behind = machine_pos + direction.0;

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&behind)
            && acceptable_items.can_accept(item_type)
        {
            info!("Transferring item Picker-upper -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Picker-upper -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&behind.as_world_pos()));
        }
    }
}
