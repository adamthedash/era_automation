use bevy::prelude::*;

use crate::{
    ground_items::GroundItemBundle,
    items::ItemType,
    map::{TilePos, WorldPos},
    player::{HeldBy, HeldItemBundle, Holding, Player},
    resources::{ResourceAmount, ResourceMarker, ResourceNodeLUT, ResourceNodeType},
    sprites::{GetSprite, SpriteSheets},
};

use super::components::*;

/// Advance the state of the harvesters if there is a resource beside it
pub fn tick_harvesters(
    machines: Query<
        (
            &TilePos,
            &mut HarvestState,
            &HarvestSpeed,
            &Direction,
            &HarvestableNodes,
        ),
        With<Harvester>,
    >,
    resource_lut: Res<ResourceNodeLUT>,
    resources: Query<(&ResourceNodeType, &ItemType), With<ResourceMarker>>,
    machine_lut: Res<MachineLUT>,
    transporters: Query<(), With<Transporter>>,
    timer: Res<Time>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (tile_pos, mut state, speed, direction, harvestable_nodes) in machines {
        // Check if there's a harvestable node in front of the machine
        let resource_pos = tile_pos + direction.0;

        let Some(resource) = resource_lut.0.get(&resource_pos) else {
            // No resource, so reset progress
            state.0 = 0.;
            continue;
        };

        // Check that resource can be harvested by this machine
        let (resource_type, item_type) = resources.get(*resource).expect("Resource node not found");
        if !harvestable_nodes.0.contains(resource_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Tick the machine
        state.0 += timer.delta_secs();

        // Check if harvest has been completed
        if state.0 >= speed.0 {
            state.0 -= speed.0;

            let behind = tile_pos - direction.0;

            // Spawn an item
            let item = commands
                .spawn((
                    // Game data
                    *item_type,
                    ResourceAmount(1),
                ))
                .id();
            item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

            // Check if there's something beside it
            if let Some(adjacent_machine) = machine_lut.0.get(&behind) {
                // Put item in the machine next to it
                // TODO: Some abstraction over "machine that can accept items"
                if transporters.contains(*adjacent_machine) {
                    info!("Transferring item Harvester -> transporter");
                    // Another transporter
                    commands
                        .entity(item)
                        .insert(TransportedItemBundle::new(*adjacent_machine, direction));
                } else {
                    // Different type of machine
                    todo!()
                }
            } else {
                info!("Transferring item Harvester -> ground");
                // Drop item on ground
                commands
                    .entity(item)
                    .insert(GroundItemBundle::new(&behind.as_world_pos()));
            }
        }
    }
}

/// Place a machine at the player's feet
pub fn place_machine(
    player: Single<(&WorldPos, &Holding), With<Player>>,
    held_machines: Query<(Entity, &Machine), With<HeldBy>>,
    mut machines: ResMut<MachineLUT>,
    resources: Res<ResourceNodeLUT>,
    mut commands: Commands,
) {
    let tile_pos = (player.0 + Vec2::splat(0.5)).tile();

    if machines.0.contains_key(&tile_pos) {
        // Machine already here
        return;
    }
    if resources.0.contains_key(&tile_pos) {
        // Resource already here
        return;
    }

    let Some((machine, machine_type)) = player
        .1
        .iter()
        .find_map(|entity| held_machines.get(entity).ok())
    else {
        // Player isn't holding a machine
        return;
    };

    machines.0.insert(tile_pos, machine);

    // Place the machine
    commands
        .entity(machine)
        // Remove heldby stuff
        .remove::<HeldItemBundle>();

    // Add placed machine stuff
    match machine_type {
        Machine::Harvester => {
            commands
                .entity(machine)
                .insert(PlacedHarvesterBundle::new(tile_pos, -IVec2::X));
        }
        Machine::Transporter => {
            commands
                .entity(machine)
                .insert(PlacedTransporterBundle::new(tile_pos, IVec2::X));
        }
    }
}

/// Move items along the transporter
pub fn tick_transporters(
    mut transported_items: Query<
        (Entity, &mut Transform, &mut TransportState),
        With<TransportedItem>,
    >,
    transporters: Query<(&TransportSpeed, &Direction, &Children, &TilePos), With<Transporter>>,
    machine_lut: Res<MachineLUT>,
    timer: Res<Time>,
    mut commands: Commands,
) {
    for (speed, direction, children, machine_pos) in transporters {
        for child in children {
            let Ok((item, mut transform, mut state)) = transported_items.get_mut(*child) else {
                // Non-item child
                continue;
            };

            // Move item along the current transporter
            state.0 += timer.delta_secs();
            let progress = state.0 / speed.0;

            let offset = progress - 0.5;
            transform.translation =
                (direction.0.as_vec2() * offset).extend(transform.translation.z);

            // Check if the item has gone off the end
            if progress >= 1. {
                // Check if there's another transporter next to it
                let adjacent_pos = machine_pos + direction.0;
                if let Some(adjacent_machine) = machine_lut.0.get(&adjacent_pos) {
                    // Put item in the machine next to it
                    if let Ok((_, direction, _, _)) = transporters.get(*adjacent_machine) {
                        // Another transporter
                        commands
                            .entity(item)
                            .insert(TransportedItemBundle::new(*adjacent_machine, direction));
                    } else {
                        // Different type of machine
                        todo!()
                    }
                } else {
                    // Drop item on ground
                    commands.entity(item).remove::<TransportedItemBundle>();

                    commands
                        .entity(item)
                        .insert(GroundItemBundle::new(&adjacent_pos.as_world_pos()));
                }
            }
        }
    }
}

/// Cycle through the sprites as the machine makes progress
pub fn animate_machine(
    machines: Query<
        (
            Entity,
            &HarvestState,
            &HarvestSpeed,
            &Children,
            &AnimationSprites,
        ),
        With<Harvester>,
    >,
    sprite_entities: Query<(), With<Sprite>>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (machine, state, speed, children, sprites) in machines {
        let sprite_index = (state.0 / (speed.0 / sprites.0.len() as f32)) as usize;

        // Delete old sprite
        // TODO: Only advance sprite when it changes
        let sprite = children
            .iter()
            .find(|child| sprite_entities.get(*child).is_ok())
            .expect("Machine has no sprite child");

        commands.entity(sprite).despawn();

        // Add new sprite
        sprites.0[sprite_index].spawn_sprite(&mut commands, &sprite_sheets, Some(machine));
    }
}
