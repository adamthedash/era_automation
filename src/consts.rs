use bevy::math::UVec2;

/// Tiles per second
pub const PLAYER_SPEED: f32 = 15.;

/// Size tiles are stored in the sprite sheet
pub const TILE_RAW_SIZE: UVec2 = UVec2::splat(16);
// How many pixels per world unit
pub const CAMERA_ZOOM: f32 = 64.;

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
pub const Z_GROUND_ITEM: f32 = 2.;
pub const Z_PLAYER: f32 = 3.;
/// Z relative to Z_PLAYER
pub const Z_HELD_ITEM: f32 = 1e-1;
/// Z relative to Z_HELD_ITEM
pub const Z_CONTAINED_ITEM: f32 = -1e-2;
/// Z relative to Z_RESOURCES
pub const Z_TRANSPORTED_ITEM: f32 = 1e-1;

/// Tiles around the starting point which can't be water terrain
pub const TERRAIN_STARTING_RADIUS: i32 = 25;
/// Tiles around the starting point which can't spawn resource nodes
pub const RESOURCE_STARTING_RADIUS: i32 = 5;

/// How much the player picks up at once
pub const RESOURCE_PICKUP_AMOUNT: usize = 1;
/// How much a resource node spawns with
pub const RESOURCE_SPAWN_AMOUNT: usize = 2;

/// Amount items bob up/down
pub const GROUND_ITEM_BOB_HEIGHT: f32 = 0.5;
/// Seconds per bob cycle
pub const GROUND_ITEM_BOB_SPEED: f32 = 2.;

/// How much a sprite is scaled up when being highlighted
pub const HIGHLIGHT_SCALE: f32 = 1.2;

/// How fast items roll down a hill.
pub const ITEM_ROLL_SPEED: f32 = 5.;
/// Minimum slope required for something to roll
pub const ROLL_FRICTION: f32 = 0.1;
