use std::ops::Deref;

use bevy::{platform::collections::HashMap, prelude::*};

use super::bundles::*;
pub use super::{
    harvester::components::*, network::components::*, picker_upper::components::*,
    transporter::components::*, windmill::components::*,
};
use crate::{items::ItemType, map::TilePos, sprites::EntitySprite, utils::query::LUTParam};

/// Marker for machines, also machine type
#[derive(Component, Debug)]
pub enum Machine {
    VillageCentre,
    Harvester,
    Transporter,
    PickerUpper,
    Windmill,
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
            Windmill => {
                commands.insert(PlacedWindmillBundle::new(pos, direction));
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
            Windmill => {
                commands.remove::<PlacedWindmillBundle>();
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

/// Maximum work rate of the machine when fully powered (actions per second).
#[derive(Component)]
pub struct MachineSpeed(pub f32);

/// Maximum power the machine can consume per second. Units match `CurrentEnergy`.
#[derive(Component)]
pub struct PowerConsumption(pub f32);

/// How much progress the machine has made towards completing one action.
/// 0 - 1
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

/// Request to transfer an item into a machine
#[derive(Message)]
pub struct TransferItem {
    /// Item should be "in limbo"
    pub item: Entity,
    /// Machine should be placed & accepting items
    pub target_machine: Entity,
}

/// Current energy produced per second by windmills (amount available to adjacent machines).
#[derive(Component)]
pub struct PowerProduction(pub f32);
