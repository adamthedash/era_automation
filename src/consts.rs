use bevy::math::UVec2;

/// Tiles per second
pub const PLAYER_SPEED: f32 = 4.;

/// Size tiles are stored in the sprite sheet
pub const TILE_RAW_SIZE: UVec2 = UVec2::splat(16);
/// Size tiles are displayed on screen
pub const TILE_DISPLAY_SIZE: UVec2 = UVec2::splat(32);

/// Number of tiles per chunk
pub const CHUNK_SIZE: UVec2 = UVec2::splat(16);

/// Resource density per tile
pub const RESOURCE_DENSITY_LOG: f32 = 1. / 16.;
pub const RESOURCE_DENSITY_BUSH: f32 = 1. / 16.;

/// Number of chunks around the player to load
pub const CHUNK_LOAD_RADIUS: i32 = 3;

/// Player's interact radius in world units
pub const PLAYER_REACH: f32 = 2.;

/// Z layers for sprite rendering
pub const Z_TERRAIN: f32 = 0.;
pub const Z_RESOURCES: f32 = 1.;
pub const Z_PLAYER: f32 = 2.;
