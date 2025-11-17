use std::f32::consts::FRAC_PI_2;

use bevy::{math::ops::atan2, prelude::*};

use super::components::*;
use crate::{
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

// ==============================
// Electricity-network debug
// ==============================

/// Toggle electricity-network debug labels on/off.
///
/// This function flips the `ElectricityDebugEnabled` resource. It is intended to
/// be bound to the `L` key via the plugin registration.
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

/// Spawn electricity debug labels for all placed machines that don't already
/// have a label in the LUT.
///
/// Each label is a 2D text entity positioned over the machine. The label's
/// text contains:
/// - network index the machine belongs to (if any)
/// - requested power (consumer machines)
/// - produced power (producer machines)
/// - actual power supplied by the network
pub fn spawn_electricity_debug(
    machines: Query<
        (
            &TilePos,
            &Machine,
            Option<&PowerConsumption>,
            Option<&PowerProduction>,
        ),
        With<Placed>,
    >,
    energy_networks: Res<EnergyNetworks>,
    mut lut: ResMut<ElectricityDebugLUT>,
    mut commands: Commands,
) {
    // Load the font once per invocation and reuse it for spawned Text2d children.
    for (tile_pos, _machine, maybe_consumption, maybe_production) in &machines {
        // Skip if a label already exists for this machine.
        if lut.0.contains_key(tile_pos) {
            continue;
        }

        // Determine values to render
        let network_index = energy_networks.membership.get(tile_pos).copied();
        let requested = energy_networks
            .power_demands
            .get(tile_pos)
            .copied()
            .unwrap_or(0.0);
        let produced = maybe_production.map(|p| p.0).unwrap_or(0.0);
        let supplied = energy_networks
            .power_provided
            .get(tile_pos)
            .copied()
            .unwrap_or(0.0);

        // Format a multi-line label
        let mut label_text = String::new();
        if let Some(idx) = network_index {
            label_text.push_str(&format!("N: {}\n", idx));
        } else {
            label_text.push_str("N: -\n");
        }

        // Only show requested if this is a consumer
        if maybe_consumption.is_some() {
            label_text.push_str(&format!("Req: {:.2}\n", requested));
        } else {
            label_text.push_str("Req: -\n");
        }

        // Only show produced if this is a producer
        if maybe_production.is_some() {
            label_text.push_str(&format!("Prod: {:.2}\n", produced));
        } else {
            label_text.push_str("Prod: -\n");
        }

        label_text.push_str(&format!("Sup: {:.2}", supplied));

        // Spawn a parent entity with transform & marker, and a child Text component
        let transform = tile_pos.as_transform(10. + 0.2);
        const FONT_SIZE: f32 = 32.;

        let parent_entity = commands
            .spawn((
                ElectricityDebugLabel,
                *tile_pos,
                transform,
                Visibility::Inherited,
            ))
            .with_children(|parent| {
                // Spawn a world-space `Text2d` child using the loaded font and a small size.
                // Center the text and scale the child so the text area matches a single tile.
                parent.spawn((
                    Text2d::new(label_text.clone()),
                    TextFont {
                        font_size: FONT_SIZE,
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_scale(Vec2::splat(1. / (FONT_SIZE * 4.)).extend(1.)),
                ));
            })
            .id();

        lut.0.insert(*tile_pos, parent_entity);
    }
}

/// Update existing electricity debug labels with the latest network values.
///
/// This system writes only the text content of existing label entities. It uses
/// the LUT to find the label entity for each placed machine and updates the
/// text accordingly.
pub fn update_electricity_debug(
    mut labels: Query<(Entity, &TilePos, &Children), With<ElectricityDebugLabel>>,
    mut commands: Commands,
    energy_networks: Res<EnergyNetworks>,
    machines: Query<
        (
            &TilePos,
            Option<&PowerConsumption>,
            Option<&PowerProduction>,
        ),
        With<Placed>,
    >,
) {
    // Build a small lookup for machine-side quick access
    let mut machine_info: std::collections::HashMap<TilePos, (Option<f32>, Option<f32>)> =
        std::collections::HashMap::new();
    for (tile_pos, maybe_consumption, maybe_production) in &machines {
        let req = maybe_consumption.map(|c| c.0);
        let prod = maybe_production.map(|p| p.0);
        machine_info.insert(*tile_pos, (req, prod));
    }

    for (parent_entity, tile_pos, children) in &mut labels {
        // Get latest values
        let network_index = energy_networks.membership.get(tile_pos).copied();
        let requested = energy_networks
            .power_demands
            .get(tile_pos)
            .copied()
            .unwrap_or(0.0);
        let supplied = energy_networks
            .power_provided
            .get(tile_pos)
            .copied()
            .unwrap_or(0.0);

        let (maybe_req_component, maybe_prod_component) =
            machine_info.get(tile_pos).copied().unwrap_or((None, None));

        let produced = maybe_prod_component.unwrap_or(0.0);

        let mut label_text = String::new();
        if let Some(idx) = network_index {
            label_text.push_str(&format!("N: {}\n", idx));
        } else {
            label_text.push_str("N: -\n");
        }

        if maybe_req_component.is_some() {
            label_text.push_str(&format!("Req: {:.2}\n", requested));
        } else {
            label_text.push_str("Req: -\n");
        }

        if maybe_prod_component.is_some() {
            label_text.push_str(&format!("Prod: {:.2}\n", produced));
        } else {
            label_text.push_str("Prod: -\n");
        }

        label_text.push_str(&format!("Sup: {:.2}", supplied));

        // Remove existing child text entities and spawn a new `Text2d` child with updated text.
        // Collect child entities into an owned Vec<Entity> first to avoid borrowing issues
        // while mutating the world (despawning children).
        for child in children.iter() {
            commands.entity(child).despawn();
        }

        const FONT_SIZE: f32 = 32.;
        commands.entity(parent_entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(label_text.clone()),
                TextFont {
                    font_size: FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::WHITE),
                // Scale text so the block is the same size as a tile
                Transform::from_scale(Vec2::splat(1. / (FONT_SIZE * 4.)).extend(1.)),
            ));
        });
    }
}
