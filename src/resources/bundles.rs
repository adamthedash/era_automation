use bevy::prelude::*;

use super::components::*;

#[derive(Bundle)]
pub struct ResourceNodeBundle {
    node_type: ResourceNodeType,
    marker: ResourceMarker,
    amount: ResourceAmount,
    amount_max: ResourceMaxAmount,
    regen_rate: ResourceRegenRate,
    regen_state: ResourceRegenState,
}

impl ResourceNodeBundle {
    pub fn new(
        node_type: ResourceNodeType,
        amount: usize,
        amount_max: usize,
        regen_rate: f32,
    ) -> Self {
        Self {
            node_type,
            marker: ResourceMarker,
            amount: ResourceAmount(amount),
            amount_max: ResourceMaxAmount(amount_max),
            regen_rate: ResourceRegenRate(regen_rate),
            regen_state: ResourceRegenState(0.),
        }
    }
}
