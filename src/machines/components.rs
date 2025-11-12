use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::ops::Deref;

use super::bundles::*;
use crate::{
    items::ItemType,
    map::TilePos,
    resources::ResourceNodeType,
    sprites::{EntitySprite, TerrainSprite},
    utils::query::LUTParam,
};

/// Marker for machines, also machine type
#[derive(Component, Debug)]
pub enum Machine {
    VillageCentre,
    Harvester,
    Transporter,
    PickerUpper,
    Windmill,
}

impl Machine {
    /// Add components for when machines are placed down
    pub fn place(&self, commands: &mut EntityCommands, pos: TilePos, direction: IVec2) {
        use Machine::*;
        match self {
            Harvester => {
                commands.insert(PlacedHarvesterBundle::new(pos, direction));
            }
            Transporter => {
                commands.insert(PlacedTransporterBundle::new(pos, direction));
            }
            PickerUpper => {
                commands.insert(PlacedPickerUpperBundle::new(pos, direction));
            }
            Windmill => {
                commands.insert(PlacedWindmillBundle::new(pos, direction));
            }
            VillageCentre => unreachable!("Village centre cannot be placed"),
        }
    }

    pub fn unplace(&self, commands: &mut EntityCommands) {
        use Machine::*;
        match self {
            Harvester => {
                commands.remove::<PlacedHarvesterBundle>();
            }
            Transporter => {
                commands.remove::<PlacedTransporterBundle>();
            }
            PickerUpper => {
                commands.remove::<PlacedPickerUpperBundle>();
            }
            Windmill => {
                commands.remove::<PlacedWindmillBundle>();
            }
            VillageCentre => unreachable!("Village centre cannot be placed"),
        }
    }
}

/// Marker for a machine that can have items given to it
#[derive(Component)]
pub enum AcceptsItems {
    Any,
    Whitelist(Vec<ItemType>),
}
impl AcceptsItems {
    pub fn can_accept(&self, item: &ItemType) -> bool {
        use AcceptsItems::*;
        match self {
            Any => true,
            Whitelist(item_types) => item_types.contains(item),
        }
    }
}

/// The direction the machine is facing towards
#[derive(Component)]
pub struct Direction(pub IVec2);

/// Maximum work rate of the machine when fully powered (actions per second).
#[derive(Component)]
pub struct MachineSpeed(pub f32);

/// Maximum power the machine can consume per second. Units match `CurrentEnergy`.
#[derive(Component)]
pub struct PowerConsumption(pub f32);

/// How much progress the machine has made towards completing one action.
/// 0 - 1
#[derive(Component, Default)]
pub struct MachineState(pub f32);

#[derive(Resource, Default)]
pub struct MachineLUT(pub HashMap<TilePos, Entity>);

impl Deref for MachineLUT {
    type Target = HashMap<TilePos, Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Machines<'w, 's, Q, F> = LUTParam<'w, 's, MachineLUT, TilePos, Q, F>;

/// Resource containing connected networks of placed machines.
/// Each network is represented as a set of `TilePos` that are adjacent (4-connected).
#[derive(Resource, Default)]
pub struct EnergyNetworks(pub Vec<HashSet<TilePos>>);

/// Sprites which are cycled through depending on the progress of the machine
#[derive(Component)]
pub struct AnimationSprites(pub Vec<EntitySprite>);

/// Marker for machines placed in the world
#[derive(Component)]
pub struct Placed;

/// Marker for harvesting machines
#[derive(Component)]
pub struct Harvester;

/// Types of resources this machine can harvest
#[derive(Component)]
pub struct HarvestableNodes(pub HashSet<ResourceNodeType>);

/// Types of terrain this machine can harvest
#[derive(Component)]
pub struct HarvestableTerrain(pub HashSet<TerrainSprite>);

/// Marker for machines which transport from one tile to the next
#[derive(Component)]
pub struct Transporter;

/// Relationship for an item being transported by a transporter
#[derive(Component)]
#[relationship(relationship_target = Transporting)]
pub struct TransportedBy(pub Entity);

/// Relationship for a transporter
#[derive(Component)]
#[relationship_target(relationship = TransportedBy)]
pub struct Transporting(Vec<Entity>);

/// Marker for picker-upper machines
#[derive(Component)]
pub struct PickerUpper;

/// Request to transfer an item into a machine
#[derive(Message)]
pub struct TransferItem {
    /// Item should be "in limbo"
    pub item: Entity,
    /// Machine should be placed & accepting items
    pub target_machine: Entity,
}

/// Current energy produced per second by windmills (amount available to adjacent machines).
#[derive(Component)]
pub struct CurrentEnergy(pub f32);

/// Marker for windmill machines
#[derive(Component)]
pub struct Windmill;
