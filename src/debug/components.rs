use bevy::{platform::collections::HashMap, prelude::*};

use crate::map::TilePos;

#[derive(Component)]
pub struct GradientArrow;

#[derive(Resource, Default)]
pub struct GradientArrowLUT(pub HashMap<TilePos, Entity>);

#[derive(Resource, Default, PartialEq, Eq)]
pub struct GradientArrowsEnabled(pub bool);
