use std::iter::FromIterator;

use bevy::{platform::collections::HashSet, prelude::*};

use super::super::components::*;
use crate::{
    consts::Z_RESOURCES,
    map::TilePos,
    player::Targettable,
    resources::ResourceNodeType,
    sprites::{EntitySprite, TerrainSprite},
};

/// For harvester machines at all times
#[derive(Bundle)]
pub struct HarvesterBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    harvester_marker: Harvester,
    harvestable_nodes: HarvestableNodes,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
}
impl HarvesterBundle {
    pub fn new(
        speed: f32,
        power_consumption: f32,
        harvestable_nodes: impl IntoIterator<Item = ResourceNodeType>,
        sprites: Vec<EntitySprite>,
    ) -> Self {
        Self {
            machine_marker: Machine::Harvester,
            harvester_marker: Harvester,
            harvestable_nodes: HarvestableNodes(HashSet::from_iter(harvestable_nodes)),
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
            animation_sprites: AnimationSprites(sprites),
        }
    }
}

/// For harvester machines when placed down
#[derive(Bundle)]
pub struct PlacedHarvesterBundle {
    tile_pos: TilePos,
    transform: Transform,
    state: MachineState,
    output_direction: Direction,
    placed: Placed,
    targettable: Targettable,
}
impl PlacedHarvesterBundle {
    pub fn new(tile_pos: TilePos, output_direction: IVec2) -> Self {
        Self {
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            output_direction: Direction(output_direction),
            state: MachineState(0.),
            placed: Placed,
            targettable: Targettable,
        }
    }
}

#[derive(Bundle)]
pub struct WaterWheelBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    harvester_marker: Harvester,
    harvestable_terrain: HarvestableTerrain,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
}
impl WaterWheelBundle {
    pub fn new(
        speed: f32,
        power_consumption: f32,
        harvestable_terrain: impl IntoIterator<Item = TerrainSprite>,
        sprites: Vec<EntitySprite>,
    ) -> Self {
        Self {
            machine_marker: Machine::Harvester,
            animation_sprites: AnimationSprites(sprites),
            harvester_marker: Harvester,
            harvestable_terrain: HarvestableTerrain(HashSet::from_iter(harvestable_terrain)),
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
        }
    }
}
