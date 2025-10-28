use crate::{map::WorldPos, player::Player};

use super::components::*;
use bevy::prelude::*;
use rand::{random_bool, random_range};

/// Spawn some fluff particles to show wind direction
pub fn spawn_fluff(
    player: Single<&WorldPos, With<Player>>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    let fluffs_per_second = 1.;

    if random_bool(timer.delta_secs_f64() * fluffs_per_second) {
        let pos = *player + Vec2::new(random_range(-10.0..10.0), random_range(-10.0..10.0));

        commands.spawn((
            Fluff,
            Lifetime(10.),
            pos,
            pos.as_transform(10.)
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
        pos.0 += wind.direction * wind.speed * timer.delta_secs();

        // Update lifetime / despawn if needed
        lifetime.0 -= timer.delta_secs();
        if lifetime.0 <= 0. {
            commands.entity(fluff).despawn();
            continue;
        }

        // Update sprite - fade in/out
        let alpha = match lifetime.0 {
            x @ 0.0..2.0 => (x / 2.0).clamp(0., 1.),
            x @ 8.0..10.0 => ((10. - x) / 2.0).clamp(0., 1.),
            _ => 1.,
        };

        let mut sprite = children
            .first()
            .and_then(|entity| sprites.get_mut(*entity).ok())
            .expect("Fluff has no sprite!");

        sprite.color = sprite.color.with_alpha(alpha);
    }
}
