use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    container::{ContainableItems, Container},
    machines::{
        HarvesterBundle, PickerUpperBundle, TransporterBundle, WaterWheelBundle, WindmillBundle,
    },
    resources::{ResourceNodeType, ResourceType},
    sprites::{EntitySprite, GetSprite, ItemSprite, SpriteSheets, TerrainSprite},
};

/// Items that can be held / moved around
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ItemType {
    Berry,
    Log,
    Water,
    Bowl,
    BushWhacker,
    Transporter,
    PickerUpper,
    TripAxe,
    WaterWheel,
    Windmill,
}

impl ItemType {
    pub fn sprite_type(&self) -> ItemSprite {
        use ItemType::*;
        match self {
            Berry => ItemSprite::Berry,
            Log => ItemSprite::Log,
            Water => ItemSprite::Water,
            Bowl => ItemSprite::Bowl,
            BushWhacker => ItemSprite::BushWhacker,
            Transporter => ItemSprite::Transporter,
            PickerUpper => ItemSprite::PickerUpper,
            TripAxe => ItemSprite::TripAxe,
            WaterWheel => ItemSprite::WaterWheel,
            Windmill => ItemSprite::Windmill,
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
            BushWhacker => None,
            Transporter => None,
            PickerUpper => None,
            TripAxe => None,
            WaterWheel => None,
            Windmill => None,
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
            BushWhacker => {
                commands.insert(HarvesterBundle::new(
                    2.,
                    1.,
                    [ResourceNodeType::Bush],
                    vec![EntitySprite::BushWhacker1, EntitySprite::BushWhacker2],
                ));
            }
            Transporter => {
                commands.insert(TransporterBundle::new(
                    2.,
                    1.,
                    vec![EntitySprite::Transporter],
                ));
            }
            PickerUpper => {
                commands.insert(PickerUpperBundle::new(
                    2.,
                    1.,
                    vec![EntitySprite::PickerUpper],
                ));
            }
            TripAxe => {
                commands.insert(HarvesterBundle::new(
                    2.,
                    1.,
                    [ResourceNodeType::Tree],
                    vec![EntitySprite::TripAxe1, EntitySprite::TripAxe2],
                ));
            }
            WaterWheel => {
                commands.insert(WaterWheelBundle::new(
                    2.,
                    1.,
                    [TerrainSprite::Water],
                    vec![EntitySprite::WaterWheel1, EntitySprite::WaterWheel2],
                ));
            }
            Windmill => {
                commands.insert(WindmillBundle::new(
                    2.,
                    vec![EntitySprite::Windmill1, EntitySprite::Windmill2],
                ));
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
