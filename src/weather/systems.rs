use std::f32::consts::{FRAC_PI_2, PI};

use crate::{
    consts::{
        FLUFF_FADE, FLUFF_LIFETIME, FLUFFS_PER_SECOND, WIND_CHANGES_PER_SECOND,
        WIND_TRANSITION_TIME, Z_WEATHER,
    },
    map::WorldPos,
    player::Player,
};

use super::components::*;
use bevy::prelude::*;
use rand::{random_bool, random_range};

/// Spawn some fluff particles to show wind direction
pub fn spawn_fluff(
    player: Single<&WorldPos, With<Player>>,
    wind: Res<Wind>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    if random_bool(timer.delta_secs_f64() * FLUFFS_PER_SECOND as f64) {
        // Spawn particles upstream of wind so they blow across the screen
        let v1 = -wind.direction_vec();
        let v2 = Vec2::from_angle(FRAC_PI_2).rotate(v1);

        let particle_spawn_radius = 20.;
        let pos = *player
            + v1 * random_range(0.0..particle_spawn_radius)
            + v2 * random_range(-particle_spawn_radius..particle_spawn_radius);

        commands.spawn((
            Fluff,
            Lifetime(FLUFF_LIFETIME),
            pos,
            pos.as_transform(Z_WEATHER)
                .with_scale(Vec2::splat(0.1).extend(1.)),
            Visibility::Inherited,
            children![(
                Sprite::from_color(Color::WHITE.with_alpha(0.), Vec2::splat(1.)),
                Transform::IDENTITY
            )],
        ));
    }
}

/// Move fluffs around, despawn them
pub fn tick_fluffs(
    fluffs: Query<(Entity, &mut WorldPos, &mut Lifetime, &Children), With<Fluff>>,
    mut sprites: Query<&mut Sprite>,
    wind: Res<Wind>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    for (fluff, mut pos, mut lifetime, children) in fluffs {
        // Move em
        pos.0 += wind.velocity() * timer.delta_secs();

        // Update lifetime / despawn if needed
        lifetime.0 -= timer.delta_secs();
        if lifetime.0 <= 0. {
            commands.entity(fluff).despawn();
            continue;
        }

        // Update sprite - fade in/out
        const FADE_END: f32 = FLUFF_LIFETIME - FLUFF_FADE;
        let alpha = match lifetime.0 {
            // Fade in
            x @ 0.0..FLUFF_FADE => (x / FLUFF_FADE).clamp(0., 1.),
            // Fade out
            x @ FADE_END..FLUFF_LIFETIME => ((FLUFF_LIFETIME - x) / FLUFF_FADE).clamp(0., 1.),
            // Middle
            _ => 1.,
        };

        let mut sprite = children
            .first()
            .and_then(|entity| sprites.get_mut(*entity).ok())
            .expect("Fluff has no sprite!");

        sprite.color = sprite.color.with_alpha(alpha);
    }
}

/// Change the direction of the wind gradually
pub fn change_wind_direction(
    wind_delta: Option<ResMut<WindDelta>>,
    mut wind: ResMut<Wind>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    if let Some(mut wind_delta) = wind_delta {
        // Lerp wind towards the target
        let delta_time = wind_delta.time_left.min(timer.delta_secs());

        wind.direction = (wind.direction + wind_delta.direction * delta_time).rem_euclid(2. * PI);
        wind.speed += wind_delta.speed * delta_time;

        wind_delta.time_left -= delta_time;
        if wind_delta.time_left == 0. {
            // Delta finished applying, remove it
            commands.remove_resource::<WindDelta>();
        }
    } else {
        // Random chance to change wind direction
        if random_bool(WIND_CHANGES_PER_SECOND as f64 * timer.delta_secs_f64()) {
            let target_wind = Wind::random();
            info!("Changing direction to: {:?}", target_wind);

            // Always move towards shortest path
            let mut direction_delta = target_wind.direction - wind.direction;
            if direction_delta < -PI {
                direction_delta += 2. * PI;
            } else if direction_delta > PI {
                direction_delta -= 2. * PI;
            }

            commands.insert_resource(WindDelta {
                direction: direction_delta / WIND_TRANSITION_TIME,
                speed: (target_wind.speed - wind.speed) / WIND_TRANSITION_TIME,
                time_left: WIND_TRANSITION_TIME,
            });
        }
    }
}
