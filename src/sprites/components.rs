use bevy::prelude::*;

use crate::{consts::TILE_RAW_SIZE, items::ItemType};

/// Indexes into terrain_sprites.png
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Hash, Debug)]
#[repr(usize)]
pub enum TerrainSprite {
    #[default]
    Grass,
    Water,
    Dirt,
    Rock,
    Snow,
}

impl TerrainSprite {
    pub fn item_type(&self) -> Option<ItemType> {
        use TerrainSprite::*;
        match self {
            Grass => None,
            Water => Some(ItemType::Water),
            Dirt => None,
            Rock => None,
            Snow => None,
        }
    }
}

/// Indexes into resource_sprites.png
#[derive(Component, Clone, Copy, Debug)]
#[repr(usize)]
pub enum ResourceSprite {
    Tree,
    TreeDepleted,
    Bush,
    BushDepleted,
    House,
    DebugArrow,
}

/// Indexes into entity_sheet.png
#[derive(Component, Clone, Copy)]
#[repr(usize)]
pub enum EntitySprite {
    Player,
    BushWhacker1,
    BushWhacker2,
    Transporter,
    PickerUpper,
    TripAxe1,
    TripAxe2,
    WaterWheel1,
    WaterWheel2,
    Windmill1,
    Windmill2,
}

/// Indexes into item_sheet.png
#[derive(Component, Clone, Copy)]
#[repr(usize)]
pub enum ItemSprite {
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

/// Holds a spritesheet image & layout info
pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Handles for all of the sprite sheets
#[derive(Resource)]
pub struct SpriteSheets {
    /// resource_sheet.png
    pub resources: SpriteSheet,
    /// entity_sheet.png
    pub entities: SpriteSheet,
    /// item_sheet.png
    pub items: SpriteSheet,
}

// ============================================
// Impls
// ============================================

pub trait GetSprite {
    /// Get a sprite handle that can be added to an entity
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite;

    /// Spawn a sprite with a transform to normalise it into unit square
    fn spawn_sprite(
        &self,
        commands: &mut Commands,
        sprite_sheets: &SpriteSheets,
        parent: Option<Entity>,
    ) -> Entity {
        let mut entity = commands.spawn((
            self.get_sprite(sprite_sheets),
            Transform::from_scale(1. / TILE_RAW_SIZE.as_vec2().extend(1.)),
        ));

        if let Some(parent) = parent {
            entity.insert(ChildOf(parent));
        }

        entity.id()
    }
}

impl From<ResourceSprite> for usize {
    fn from(val: ResourceSprite) -> Self {
        val as usize
    }
}

impl GetSprite for ResourceSprite {
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite {
        Sprite {
            image: sprite_sheets.resources.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheets.resources.layout.clone(),
                index: (*self).into(),
            }),
            ..Default::default()
        }
    }
}

impl From<EntitySprite> for usize {
    fn from(val: EntitySprite) -> Self {
        val as usize
    }
}

impl GetSprite for EntitySprite {
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite {
        Sprite {
            image: sprite_sheets.entities.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheets.entities.layout.clone(),
                index: (*self).into(),
            }),
            ..Default::default()
        }
    }
}

impl From<ItemSprite> for usize {
    fn from(val: ItemSprite) -> Self {
        val as usize
    }
}

impl GetSprite for ItemSprite {
    fn get_sprite(&self, sprite_sheets: &SpriteSheets) -> Sprite {
        Sprite {
            image: sprite_sheets.items.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheets.items.layout.clone(),
                index: (*self).into(),
            }),
            ..Default::default()
        }
    }
}
