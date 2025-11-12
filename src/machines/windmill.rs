use bevy::prelude::*;

use crate::machines::{Direction, EnergyStored, Windmill};
use crate::weather::Wind;

/// Tick all placed windmills and add produced energy to their `EnergyStored` component.
pub fn tick_windmills(
    wind: Res<Wind>,
    timer: Res<Time>,
    mut windmills: Query<(&Direction, &mut EnergyStored), With<Windmill>>,
) {
    for (direction, mut energy) in windmills.iter_mut() {
        // Compute alignment in [-1, 1]; only positive alignment produces energy.
        let alignment = direction.0.as_vec2().dot(wind.direction_vec()).max(0.0);

        // Produced energy this tick (wind.speed is units-per-second).
        let produced = wind.speed * alignment * timer.delta_secs();

        energy.0 += produced;
    }
}
