use std::f32::consts::FRAC_PI_2;

use bevy::{math::ops::atan2, prelude::*};

use super::components::*;
use crate::{
    consts::{DEBUG_FONT_RENDER_SIZE, Z_DEBUG},
    machines::{EnergyNetworks, Machine, Placed, PowerConsumption, PowerProduction},
    map::{ChunkPos, GradientData, TilePos},
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

/// Toggle whether gradient arrows are enabled.
pub fn toggle_gradient_arrows(mut enabled: ResMut<GradientArrowsEnabled>) {
    enabled.0 ^= true;
    if enabled.0 {
        info!("Showing gradient arrows");
    } else {
        info!("Hiding gradient arrows");
    }
}

/// Despawn all gradient arrow entities and clear the LUT.
pub fn despawn_gradient_arrows(mut arrows: ResMut<GradientArrowLUT>, mut commands: Commands) {
    for (_, arrow) in arrows.0.drain() {
        commands.entity(arrow).despawn();
    }
}

/// Spawn gradient-arrow entities for each tile in every loaded chunk that
/// doesn't already have an arrow entity in the LUT.
pub fn spawn_gradient_arrows(
    chunks: Query<(&ChunkPos, &GradientData)>,
    mut arrows_lut: ResMut<GradientArrowLUT>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (chunk_pos, gradients) in chunks {
        let chunk_pos = chunk_pos.as_tile_pos();
        for (yo, row) in gradients.0.iter().enumerate() {
            for (xo, grad) in row.iter().enumerate() {
                let tile_pos = chunk_pos + IVec2::new(xo as i32, yo as i32);
                if !arrows_lut.0.contains_key(&tile_pos) {
                    let transform = tile_pos
                        .as_transform(10.)
                        .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2))
                        .with_scale(Vec2::splat(grad.length().sqrt()).extend(1.));

                    // Spawn new arrow
                    let arrow = commands.spawn((transform, GradientArrow)).id();
                    ResourceSprite::DebugArrow.spawn_sprite(
                        &mut commands,
                        &sprite_sheets,
                        Some(arrow),
                    );

                    arrows_lut.0.insert(tile_pos, arrow);
                }
            }
        }
    }
}

/// Update cached gradient-arrow transforms when the chunk's gradient data changes.
pub fn update_gradient_arrows(
    chunks: Query<(&ChunkPos, &GradientData), Changed<GradientData>>,
    arrows_lut: ResMut<GradientArrowLUT>,
    mut commands: Commands,
) {
    for (chunk_pos, gradients) in chunks {
        info!("Updating gradient arrows for chunk: {:?}", chunk_pos.0);
        let chunk_tile_pos = chunk_pos.as_tile_pos();
        for (yo, row) in gradients.0.iter().enumerate() {
            for (xo, grad) in row.iter().enumerate() {
                let tile_pos = chunk_tile_pos + IVec2::new(xo as i32, yo as i32);

                let entity = arrows_lut
                    .0
                    .get(&tile_pos)
                    .expect("Arrow entity should already exist at this point");

                let transform = tile_pos
                    .as_transform(10.)
                    .with_rotation(Quat::from_rotation_z(atan2(grad.y, grad.x) + FRAC_PI_2))
                    .with_scale(Vec2::splat(grad.length().sqrt()).extend(1.));

                commands.entity(*entity).insert(transform);
            }
        }
    }
}

/// Toggle electricity-network debug labels on/off.
pub fn toggle_electricity_debug(mut enabled: ResMut<ElectricityDebugEnabled>) {
    enabled.0 ^= true;
    if enabled.0 {
        info!("Showing electricity debug labels");
    } else {
        info!("Hiding electricity debug labels");
    }
}

/// Despawn all electricity debug label entities and clear the LUT.
pub fn despawn_electricity_debug(mut lut: ResMut<ElectricityDebugLUT>, mut commands: Commands) {
    for (_, label) in lut.0.drain() {
        commands.entity(label).despawn();
    }
}

/// Update existing electricity debug labels with the latest network values.
pub fn update_electricity_debug(
    labels: Query<&Children, With<ElectricityDebugLabel>>,
    mut label_lut: ResMut<ElectricityDebugLUT>,
    energy_networks: Res<EnergyNetworks>,
    machines: Query<
        (&TilePos, Has<PowerConsumption>, Option<&PowerProduction>),
        (With<Placed>, With<Machine>),
    >,
    mut commands: Commands,
) {
    for (tile_pos, consumer, production) in &machines {
        // Network index
        let index = if let Some(idx) = energy_networks.membership.get(tile_pos) {
            format!("{idx}")
        } else {
            "-".to_string()
        };

        // Power consumption for consumers
        let requested =
            if consumer && let Some(requested) = energy_networks.power_demands.get(tile_pos) {
                format!("{requested:.2}")
            } else {
                "-".to_string()
            };

        // Power production for producers
        let production = if let Some(production) = production {
            format!("{:.2}", production.0)
        } else {
            "-".to_string()
        };

        // Actual power supplied by network for consumers
        let supplied = if let Some(supplied) = energy_networks.power_provided.get(tile_pos) {
            format!("{:.2}", supplied)
        } else {
            "-".to_string()
        };

        let debug_lines = [
            format!("N: {index}"),
            format!("P: {production}"),
            format!("R: {requested}"),
            format!("S: {supplied}"),
        ];

        // Get the label entity, or spawn one if it doesn't exist
        let label = if let Some(&label) = label_lut.0.get(tile_pos) {
            // Label exists, so clear old Text2d
            let text2d = labels
                .get(label)
                .expect("Label entity doesn't exist!")
                .first()
                .expect("Text2d component doesn't exist!");
            commands.entity(*text2d).despawn();
            label
        } else {
            // Create a new label
            let label = commands
                .spawn((
                    ElectricityDebugLabel,
                    *tile_pos,
                    tile_pos.as_transform(Z_DEBUG),
                    Visibility::Inherited,
                ))
                .id();

            label_lut.0.insert(*tile_pos, label);
            label
        };

        // Spawn new Text2d
        commands.entity(label).with_children(|parent| {
            // Spawn a world-space `Text2d` child using the loaded font and a small size.
            // Center the text and scale the child so the text area matches a single tile.
            parent.spawn((
                Text2d::new(debug_lines.join("\n")),
                TextFont {
                    font_size: DEBUG_FONT_RENDER_SIZE,
                    ..Default::default()
                },
                TextColor(Color::srgb(1., 0., 0.)),
                Transform::from_scale(
                    Vec2::splat(1. / (DEBUG_FONT_RENDER_SIZE * debug_lines.len() as f32))
                        .extend(1.),
                ),
            ));
        });
    }
}
