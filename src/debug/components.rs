use bevy::{platform::collections::HashMap, prelude::*};

use crate::map::TilePos;

/// Marker component for spawned gradient arrows used for visualising terrain gradients.
#[derive(Component)]
pub struct GradientArrow;

/// Lookup table mapping a tile position to the spawned gradient-arrow entity.
#[derive(Resource, Default)]
pub struct GradientArrowLUT(pub HashMap<TilePos, Entity>);

/// Resource controlling whether gradient-arrow debug rendering is enabled.
#[derive(Resource, Default, PartialEq, Eq)]
pub struct GradientArrowsEnabled(pub bool);

/// Marker component attached to the parent entity of an electricity debug label.
///
/// The parent entity holds a `TilePos` and a `Transform`; it has a child `Text`
/// entity that renders the textual debug information.
#[derive(Component)]
pub struct ElectricityDebugLabel;

/// Lookup table mapping a machine's `TilePos` to the parent entity of its
/// electricity debug label.
///
/// The parent entity is spawned by the debug systems and is the entity that gets
/// the `ElectricityDebugLabel` component. The child `Text` entity is owned by the
/// parent via the hierarchical children relationship.
#[derive(Resource, Default)]
pub struct ElectricityDebugLUT(pub HashMap<TilePos, Entity>);

/// Resource controlling whether the electricity-network debug overlay is enabled.
///
/// When `true` the debug systems will spawn and update label entities for placed
/// machines. When `false` those entities will be despawned.
#[derive(Resource, Default, PartialEq, Eq)]
pub struct ElectricityDebugEnabled(pub bool);
