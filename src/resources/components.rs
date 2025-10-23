use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    map::TilePos,
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

/// The type of resource used by the village
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ResourceType {
    Wood,
    Food,
    Water,
}

/// The type of node resource node placed in the world
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ResourceNodeType {
    Tree,
    Bush,
    Water,
}

impl ResourceNodeType {
    /// Get the corresponding sprite type if it's available
    pub fn sprite(&self) -> ResourceSprite {
        match self {
            ResourceNodeType::Tree => ResourceSprite::Tree,
            ResourceNodeType::Bush => ResourceSprite::Bush,
            ResourceNodeType::Water => unreachable!("Water node should never be rendered"),
        }
    }
}

impl GetSprite for ResourceNodeType {
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite {
        self.sprite().get_sprite(sprite_sheets)
    }
}

/// The amount of resource left in a node
#[derive(Component)]
pub struct ResourceAmount(pub usize);

/// Sparse lookup for all resource node entities spawned in the world
#[derive(Resource, Default)]
pub struct ResourceNodeLUT(pub HashMap<TilePos, Entity>);

/// Marker for a resource node
#[derive(Component)]
pub struct ResourceMarker;
