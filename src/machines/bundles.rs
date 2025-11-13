use bevy::{platform::collections::HashSet, prelude::*};
use std::iter::FromIterator;

use super::components::*;
use crate::{
    consts::{Z_RESOURCES, Z_TRANSPORTED_ITEM},
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

/// For transporter machines at all times
#[derive(Bundle)]
pub struct TransporterBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    transporter_marker: Transporter,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
    accepts_items: AcceptsItems,
}
impl TransporterBundle {
    pub fn new(speed: f32, power_consumption: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Transporter,
            transporter_marker: Transporter,
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
            animation_sprites: AnimationSprites(sprites),
            accepts_items: AcceptsItems::Any,
        }
    }
}

/// For transporter machines when placed down
#[derive(Bundle)]
pub struct PlacedTransporterBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
}
impl PlacedTransporterBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            targettable: Targettable,
            placed: Placed,
        }
    }
}

/// For items on conveyor belts
#[derive(Bundle)]
pub struct TransportedItemBundle {
    parent: ChildOf,
    transporter: TransportedBy,
    transport_state: MachineState,
    transform: Transform,
}
impl TransportedItemBundle {
    pub fn new(transporter: Entity, transporter_direction: &Direction) -> Self {
        Self {
            parent: ChildOf(transporter),
            transporter: TransportedBy(transporter),
            transport_state: MachineState(0.),
            transform: Transform::from_translation(
                (transporter_direction.0.as_vec2() * 0.5).extend(Z_TRANSPORTED_ITEM),
            ),
        }
    }
}

/// For picker-upper machines at all times
#[derive(Bundle)]
pub struct PickerUpperBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    pickerupper_marker: PickerUpper,
    speed: MachineSpeed,
    power_consumption: PowerConsumption,
}
impl PickerUpperBundle {
    pub fn new(speed: f32, power_consumption: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::PickerUpper,
            animation_sprites: AnimationSprites(sprites),
            pickerupper_marker: PickerUpper,
            speed: MachineSpeed(speed),
            power_consumption: PowerConsumption(power_consumption),
        }
    }
}

/// For picker-upper machines when placed down
#[derive(Bundle)]
pub struct PlacedPickerUpperBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
    pickup_state: MachineState,
}
impl PlacedPickerUpperBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            placed: Placed,
            targettable: Targettable,
            pickup_state: MachineState(0.),
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

/// For windmill machines at all times (stateless parts of the prefab)
#[derive(Bundle)]
pub struct WindmillBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    windmill_marker: Windmill,
    // Speed is just used for animation for windmills
    speed: MachineSpeed,
}
impl WindmillBundle {
    pub fn new(speed: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Windmill,
            animation_sprites: AnimationSprites(sprites),
            windmill_marker: Windmill,
            speed: MachineSpeed(speed),
        }
    }
}

/// For windmill machines when placed down (stateful / instance-specific components)
#[derive(Bundle)]
pub struct PlacedWindmillBundle {
    output_direction: Direction,
    tile_pos: TilePos,
    transform: Transform,
    placed: Placed,
    targettable: Targettable,
    current_production: PowerProduction,
    // State is just used for animation for windmills
    state: MachineState,
}
impl PlacedWindmillBundle {
    pub fn new(tile_pos: TilePos, direction: IVec2) -> Self {
        Self {
            output_direction: Direction(direction),
            transform: tile_pos.as_transform(Z_RESOURCES),
            tile_pos,
            placed: Placed,
            targettable: Targettable,
            current_production: PowerProduction(0.0),
            state: MachineState(0.),
        }
    }
}
