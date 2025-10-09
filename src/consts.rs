use bevy::math::UVec2;

/// Pixels per second
pub const CAMERA_SPEED: f32 = 128.;

/// Size tiles are stored in the sprite sheet
pub const TILE_RAW_SIZE: UVec2 = UVec2::splat(16);
/// Size tiles are displayed on screen
pub const TILE_DISPLAY_SIZE: UVec2 = UVec2::splat(32);

/// Number of tiles per chunk
pub const CHUNK_SIZE: UVec2 = UVec2::splat(16);

/// Resource density per tile
pub const RESOURCE_DENSITY_LOG: f32 = 1. / 16.;
pub const RESOURCE_DENSITY_BUSH: f32 = 1. / 16.;

pub const CHUNK_LOAD_RADIUS: i32 = 3;
