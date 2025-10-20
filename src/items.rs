use bevy::prelude::*;

use crate::{
    resources::ResourceType,
    sprites::{GetSprite, ItemSprite, SpriteSheets},
};

/// The type of resource used by the village
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ItemType {
    Berry,
    Log,
    Water,
    Bowl,
}

impl ItemType {
    pub fn sprite_type(&self) -> ItemSprite {
        match self {
            ItemType::Berry => ItemSprite::Berry,
            ItemType::Log => ItemSprite::Log,
            ItemType::Water => ItemSprite::Water,
            ItemType::Bowl => ItemSprite::Bowl,
        }
    }

    /// Type of resource this item contributes to, if any
    pub fn resource_type(&self) -> Option<ResourceType> {
        match self {
            ItemType::Berry => Some(ResourceType::Food),
            ItemType::Log => Some(ResourceType::Wood),
            ItemType::Water => Some(ResourceType::Water),
            ItemType::Bowl => None,
        }
    }
}
impl GetSprite for ItemType {
    fn get_sprite(self, sprite_sheets: &SpriteSheets) -> Sprite {
        self.sprite_type().get_sprite(sprite_sheets)
    }
}
