use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{map::TilePos, resources::ResourceNodeType};

/// Marker for machines
#[derive(Component)]
pub struct Machine;

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
#[derive(Component)]
pub struct HarvestState(pub f32);

/// Types of resources this machine can harvest
#[derive(Component)]
pub struct HarvestableNodes(pub HashSet<ResourceNodeType>);

#[derive(Resource, Default)]
pub struct MachineLUT(pub HashMap<TilePos, Entity>);
