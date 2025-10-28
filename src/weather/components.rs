use std::f32::consts::PI;

use bevy::prelude::*;
use rand::random_range;

#[derive(Resource, Debug)]
pub struct Wind {
    /// Direction in radians
    pub direction: f32,
    /// Speed in tiles per second
    pub speed: f32,
}
impl Wind {
    /// Normalised direction vector
    pub fn direction_vec(&self) -> Vec2 {
        Vec2::from_angle(self.direction)
    }

    /// Velocity vector
    pub fn velocity(&self) -> Vec2 {
        self.direction_vec() * self.speed
    }

    pub fn random() -> Self {
        Self {
            direction: random_range(0.0..(2. * PI)),
            speed: random_range(0.0..3.0),
        }
    }
}

impl Default for Wind {
    fn default() -> Self {
        Self::random()
    }
}

#[derive(Resource)]
pub struct WindDelta {
    /// Direction delta per second
    pub direction: f32,
    /// Speed delta per second
    pub speed: f32,
    /// Duration over which to apply the delta
    pub time_left: f32,
}

/// Particle that floats around to indicate wind direction
#[derive(Component)]
pub struct Fluff;

/// Time left before a particle despawns
#[derive(Component)]
pub struct Lifetime(pub f32);
