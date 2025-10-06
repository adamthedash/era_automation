use bevy::math::UVec2;

/// Pixels per second
pub const CAMERA_SPEED: f32 = 128.;

/// Size tiles are displayed on screen
pub const TILE_DISPLAY_SIZE: UVec2 = UVec2::splat(32);

/// Number of tiles per chunk
pub const CHUNK_SIZE: UVec2 = UVec2::splat(16);

/// Number of sprites in the sprite sheet
pub const NUM_SPRITES: u32 = 5;
