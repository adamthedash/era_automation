use bevy::prelude::*;

#[derive(Resource)]
pub struct Wind {
    /// Normalised direction
    pub direction: Vec2,
    /// Speed in tiles per second
    pub speed: f32,
}

/// Particle that floats around to indicate wind direction
#[derive(Component)]
pub struct Fluff;

/// Time left before a particle despawns
#[derive(Component)]
pub struct Lifetime(pub f32);
