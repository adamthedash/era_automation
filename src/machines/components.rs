use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::ops::Deref;

use crate::{
    consts::{Z_RESOURCES, Z_TRANSPORTED_ITEM},
    items::ItemType,
    map::TilePos,
    player::Targettable,
    resources::ResourceNodeType,
    sprites::EntitySprite,
    utils::query::LUTParam,
};

/// Marker for machines, also machine type
#[derive(Component, Debug)]
pub enum Machine {
    VillageCentre,
    Harvester,
    Transporter,
    PickerUpper,
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

/// How long does one action take, in seconds
#[derive(Component)]
pub struct MachineSpeed(pub f32);

/// How long along an action the machine currently is
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

/// For harvester machines at all times
#[derive(Bundle)]
pub struct HarvesterBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    harvester_marker: Harvester,
    harvestable_nodes: HarvestableNodes,
    speed: MachineSpeed,
}
impl HarvesterBundle {
    pub fn new(
        speed: f32,
        harvestable_nodes: impl IntoIterator<Item = ResourceNodeType>,
        sprites: Vec<EntitySprite>,
    ) -> Self {
        Self {
            machine_marker: Machine::Harvester,
            harvester_marker: Harvester,
            harvestable_nodes: HarvestableNodes(HashSet::from_iter(harvestable_nodes)),
            speed: MachineSpeed(speed),
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
    accepts_items: AcceptsItems,
}
impl TransporterBundle {
    pub fn new(speed: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Transporter,
            transporter_marker: Transporter,
            speed: MachineSpeed(speed),
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
}
impl PickerUpperBundle {
    pub fn new(speed: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::PickerUpper,
            animation_sprites: AnimationSprites(sprites),
            pickerupper_marker: PickerUpper,
            speed: MachineSpeed(speed),
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
