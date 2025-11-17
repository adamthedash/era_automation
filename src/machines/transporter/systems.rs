use bevy::prelude::*;

use super::{super::components::*, bundles::*};
use crate::{ground_items::GroundItemBundle, items::ItemType, map::TilePos};

pub fn precheck_transporters(
    transported_items: Query<(), With<TransportedBy>>,
    transporters: Query<(&PowerConsumption, &Children, &TilePos), With<Transporter>>,
    mut energy_networks: ResMut<EnergyNetworks>,
) {
    for (power, children, machine_pos) in transporters {
        if children
            .iter()
            .any(|child| transported_items.contains(child))
        {
            // TODO: Increase power consumption per item?
            energy_networks.power_demands.insert(*machine_pos, power.0);
        }
    }
}

/// Move items along the transporter
pub fn tick_transporters(
    mut transported_items: Query<
        (Entity, &mut Transform, &mut MachineState, &ItemType),
        With<TransportedBy>,
    >,
    transporters: Query<
        (
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &Children,
            &TilePos,
        ),
        With<Transporter>,
    >,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_networks: Res<EnergyNetworks>,
    timer: Res<Time>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (speed, power, direction, children, machine_pos) in transporters {
        // Accumulate energy produced by adjacent machines (e.g., windmills)
        let energy_supply = energy_networks
            .power_provided
            .get(machine_pos)
            // Default to 0 here instead of panicing as pre-checking children is expensive
            .unwrap_or(&0.);

        // Calculate work rate based on current power supply
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        for child in children {
            let Ok((item, mut transform, mut progress, item_type)) =
                transported_items.get_mut(*child)
            else {
                // Non-item child
                continue;
            };

            // Move item along the current transporter (state.0 counts actions completed)
            progress.0 += work_rate * timer.delta_secs();

            // Items always travel along +X, as rotation is handled by machine-level transform
            transform.translation = Vec3::new(progress.0 - 0.5, 0., transform.translation.z);

            // Check if the item has gone off the end
            if progress.0 < 1. {
                // Still on the belt
                continue;
            }

            // Move item off the conveyor
            let adjacent_pos = machine_pos + direction.0;
            if let Some((machine, machine_type, acceptable_items)) = machines.get(&adjacent_pos)
                && acceptable_items.can_accept(item_type)
            {
                info!("Transferring item Transporter -> {:?}", machine_type);

                // Request to transfer to the target machine
                transfer_items.write(TransferItem {
                    item,
                    target_machine: machine,
                });
            } else {
                // Drop item on ground
                commands.entity(item).remove::<TransportedItemBundle>();

                commands
                    .entity(item)
                    .insert(GroundItemBundle::new(&adjacent_pos.as_world_pos()));
            }
        }
    }
}
