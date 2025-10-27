use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    container::{ContainableItems, Container},
    machines::{HarvesterBundle, TransporterBundle},
    resources::{ResourceNodeType, ResourceType},
    sprites::{EntitySprite, GetSprite, ItemSprite, SpriteSheets},
};

/// Items that can be held / moved around
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ItemType {
    Berry,
    Log,
    Water,
    Bowl,
    Harvester,
    Transporter,
}

impl ItemType {
    pub fn sprite_type(&self) -> ItemSprite {
        use ItemType::*;
        match self {
            Berry => ItemSprite::Berry,
            Log => ItemSprite::Log,
            Water => ItemSprite::Water,
            Bowl => ItemSprite::Bowl,
            Harvester => ItemSprite::Harvester,
            Transporter => ItemSprite::Transporter,
        }
    }

    /// Type of resource this item contributes to, if any
    pub fn resource_type(&self) -> Option<ResourceType> {
        use ItemType::*;
        match self {
            Berry => Some(ResourceType::Food),
            Log => Some(ResourceType::Wood),
            Water => Some(ResourceType::Water),
            Bowl => None,
            Harvester => None,
            Transporter => None,
        }
    }

    /// Adds extra item-specific components to an entity
    pub fn add_extra_components(&self, commands: &mut EntityCommands) {
        use ItemType::*;
        match self {
            Bowl => {
                commands.insert((
                    Container,
                    ContainableItems({
                        let mut set = HashSet::new();
                        set.insert(Water);
                        set
                    }),
                ));
            }
            Harvester => {
                commands.insert(HarvesterBundle::new(
                    2.,
                    [ResourceNodeType::Bush],
                    vec![EntitySprite::Harvester1, EntitySprite::Harvester2],
                ));
            }
            Transporter => {
                commands.insert(TransporterBundle::new(2., vec![EntitySprite::Transporter]));
            }
            _ => (),
        }
    }
}

impl GetSprite for ItemType {
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite {
        self.sprite_type().get_sprite(sprite_sheets)
    }
}
