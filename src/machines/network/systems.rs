use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use super::super::components::*;
use crate::map::TilePos;

/// Compute connected networks of placed machines and store them in the `EnergyNetworks` resource.
///
/// Each network is represented as a `HashSet<TilePos>` containing all placed-machine tile positions
/// that are 4-connected. This system currently recomputes networks on every tick.
pub fn compute_energy_networks(
    mut energy_networks: ResMut<EnergyNetworks>,
    machine_lut: Res<MachineLUT>,
) {
    // Clear previous networks and membership map
    energy_networks.networks.clear();
    energy_networks.membership.clear();

    // Visited set for TilePos values
    let mut visited = HashSet::new();

    // Iterate over all placed machines (keys in the MachineLUT)
    for pos in machine_lut.0.keys() {
        if visited.contains(pos) {
            continue;
        }

        // Start a new component
        let mut stack = vec![*pos];
        visited.insert(*pos);
        let mut component = HashSet::new();

        while let Some(cur) = stack.pop() {
            component.insert(cur);

            // Explore adjacent tile positions (4-connected)
            for neighbour in cur.adjacent() {
                if machine_lut.0.contains_key(&neighbour) && !visited.contains(&neighbour) {
                    visited.insert(neighbour);
                    stack.push(neighbour);
                }
            }
        }

        // Record the network and update membership mapping for quick lookup
        let index = energy_networks.networks.len();
        for tile in component.iter() {
            energy_networks.membership.insert(*tile, index);
        }
        energy_networks.networks.push(component);
    }
}

/// Calculate power production for each network
pub fn produce_energy(
    mut energy_networks: ResMut<EnergyNetworks>,
    energy_producers: Query<(&TilePos, &PowerProduction), With<Placed>>,
) {
    energy_networks.power_available = vec![0.; energy_networks.networks.len()];

    for (tile_pos, production) in energy_producers {
        let network = *energy_networks
            .membership
            .get(tile_pos)
            .expect("Producer has no network");
        energy_networks.power_available[network] += production.0;
    }
}

/// Distribute energy in the network according to each machine's needs
pub fn distribute_energy(mut energy_networks: ResMut<EnergyNetworks>) {
    energy_networks.power_provided = energy_networks
        .power_available
        .iter()
        .enumerate()
        // Distribute power for each network
        .flat_map(|(network, &power)| {
            // Calculcate network-level power satisfaction
            let total_demand = energy_networks.networks[network]
                .iter()
                .flat_map(|tile_pos| energy_networks.power_demands.get(tile_pos))
                .sum::<f32>();

            let satisfaction = if total_demand == 0. {
                1.
            } else {
                (power / total_demand).min(1.)
            };

            // Distribute to each consumer
            energy_networks.networks[network]
                .iter()
                .flat_map(|tile_pos| {
                    energy_networks
                        .power_demands
                        .get(tile_pos)
                        .map(|demand| (*tile_pos, demand * satisfaction))
                })
                .collect::<Vec<_>>()
        })
        .collect::<HashMap<_, _>>();
}
