use bevy::{platform::collections::HashSet, prelude::*};

use crate::{resources::ResourceNodeType, sprites::TerrainSprite};

/// Marker for harvesting machines
#[derive(Component)]
pub struct Harvester;

/// Types of resources this machine can harvest
#[derive(Component)]
pub struct HarvestableNodes(pub HashSet<ResourceNodeType>);

/// Types of terrain this machine can harvest
#[derive(Component)]
pub struct HarvestableTerrain(pub HashSet<TerrainSprite>);
