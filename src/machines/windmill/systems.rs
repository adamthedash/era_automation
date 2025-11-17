use bevy::prelude::*;

use super::super::components::*;
use crate::weather::Wind;

/// Tick all placed windmills and update their current production in the `CurrentEnergy` component.
pub fn tick_windmills(
    wind: Res<Wind>,
    timer: Res<Time>,
    windmills: Query<
        (
            &Direction,
            &mut PowerProduction,
            &MachineSpeed,
            &mut MachineState,
        ),
        With<Windmill>,
    >,
) {
    for (direction, mut current_energy, speed, mut state) in windmills {
        // Compute alignment in [-1, 1]; only positive alignment produces energy.
        let alignment = direction.0.as_vec2().dot(wind.direction_vec()).max(0.0);

        // Update energy production rate
        current_energy.0 = wind.speed * alignment;

        // Update animation
        let produced = current_energy.0 * timer.delta_secs();
        state.0 = (state.0 + produced) % speed.0;
    }
}
