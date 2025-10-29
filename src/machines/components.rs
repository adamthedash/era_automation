use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{
    consts::{Z_RESOURCES, Z_TRANSPORTED_ITEM},
    map::TilePos,
    player::Targettable,
    resources::ResourceNodeType,
    sprites::EntitySprite,
};

/// Marker for machines, also machine type
#[derive(Component, Debug)]
pub enum Machine {
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
        }
    }

    /// Give an item to this machine. Assumes the item has already been removed from where it was
    /// before and is in a "limbo" state
    pub fn give_item(&self, item: &mut EntityCommands, machine: &EntityRef) {
        assert!(
            machine.contains::<AcceptsItems>(),
            "Machine cannot accept items"
        );

        use Machine::*;
        match self {
            Transporter => {
                let direction = machine
                    .get::<Direction>()
                    .expect("Machine does not have a direction!");

                item.insert(TransportedItemBundle::new(machine.id(), direction));
            }
            _ => panic!("This machine does not accept items!"),
        }
    }
}

/// Marker for harvesting machines
#[derive(Component)]
pub struct Harvester;

/// Marker for a machine that can have items given to it
#[derive(Component)]
pub struct AcceptsItems;

/// The direction the machine is facing towards
#[derive(Component)]
pub struct Direction(pub IVec2);

/// How long does one action take, in seconds
#[derive(Component)]
pub struct MachineSpeed(pub f32);

/// How long along an action the machine currently is
#[derive(Component, Default)]
pub struct MachineState(pub f32);

/// Types of resources this machine can harvest
#[derive(Component)]
pub struct HarvestableNodes(pub HashSet<ResourceNodeType>);

#[derive(Resource, Default)]
pub struct MachineLUT(pub HashMap<TilePos, Entity>);

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

/// Sprites which are cycled through depending on the progress of the machine
#[derive(Component)]
pub struct AnimationSprites(pub Vec<EntitySprite>);

/// Marker for machines placed in the world
#[derive(Component)]
pub struct Placed;

/// Marker for picker-upper machines
#[derive(Component)]
pub struct PickerUpper;

#[derive(Message)]
pub struct TransferItem {
    /// Item should be "in limbo"
    pub item: Entity,
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
            accepts_items: AcceptsItems,
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
