use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::map::TilePos;

/// Resource containing connected networks of placed machines.
///
/// `networks` contains each network as a `HashSet<TilePos>` (4-connected).
/// `membership` maps a `TilePos` to the index in `networks` for quick lookup
/// of which network a tile belongs to.
#[derive(Resource, Default)]
pub struct EnergyNetworks {
    /// Connected components
    pub networks: Vec<HashSet<TilePos>>,
    /// LUT for network membership
    pub membership: HashMap<TilePos, usize>,
    /// Requested power from consumer machines
    pub power_demands: HashMap<TilePos, f32>,
    /// Total power available to each network
    pub power_available: Vec<f32>,
    /// Power made available to requesting machines
    pub power_provided: HashMap<TilePos, f32>,
}
