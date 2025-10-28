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
}

/// Marker for harvesting machines
#[derive(Component)]
pub struct Harvester;

/// The direction the machine is facing towards
#[derive(Component)]
pub struct Direction(pub IVec2);

/// How often an item is harvested, in seconds
#[derive(Component)]
pub struct HarvestSpeed(pub f32);

/// How long along a harvest the machine currently is
#[derive(Component, Default)]
pub struct HarvestState(pub f32);

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

/// Current progress of item being transported
#[derive(Component)]
pub struct TransportState(pub f32);

/// Total time taken for an item to be transported
#[derive(Component)]
pub struct TransportSpeed(pub f32);

/// Sprites which are cycled through depending on the progress of the machine
#[derive(Component)]
pub struct AnimationSprites(pub Vec<EntitySprite>);

/// Marker for machines placed in the world
#[derive(Component)]
pub struct Placed;

/// For harvester machines at all times
#[derive(Bundle)]
pub struct HarvesterBundle {
    machine_marker: Machine,
    animation_sprites: AnimationSprites,
    harvester_marker: Harvester,
    harvestable_nodes: HarvestableNodes,
    speed: HarvestSpeed,
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
            speed: HarvestSpeed(speed),
            animation_sprites: AnimationSprites(sprites),
        }
    }
}

/// For harvester machines when placed down
#[derive(Bundle)]
pub struct PlacedHarvesterBundle {
    tile_pos: TilePos,
    transform: Transform,
    state: HarvestState,
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
            state: HarvestState(0.),
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
    speed: TransportSpeed,
}
impl TransporterBundle {
    pub fn new(speed: f32, sprites: Vec<EntitySprite>) -> Self {
        Self {
            machine_marker: Machine::Transporter,
            transporter_marker: Transporter,
            speed: TransportSpeed(speed),
            animation_sprites: AnimationSprites(sprites),
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
    transport_state: TransportState,
    transform: Transform,
}
impl TransportedItemBundle {
    pub fn new(transporter: Entity, transporter_direction: &Direction) -> Self {
        Self {
            parent: ChildOf(transporter),
            transporter: TransportedBy(transporter),
            transport_state: TransportState(0.),
            transform: Transform::from_translation(
                (transporter_direction.0.as_vec2() * 0.5).extend(Z_TRANSPORTED_ITEM),
            ),
        }
    }
}
