use std::f32::consts::FRAC_PI_2;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    ground_items::{GroundItem, GroundItemBundle},
    items::ItemType,
    map::{TilePos, WorldPos},
    player::{HeldBy, HeldItemBundle, Holding, Player, Targetted},
    resources::{ResourceAmount, ResourceMarker, ResourceNodeLUT, ResourceNodeType},
    sprites::{GetSprite, SpriteSheets},
};

use super::components::*;

/// Advance the state of the harvesters if there is a resource beside it
pub fn tick_harvesters(
    machines: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
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

    info!("Placing machine {:?} at {:?}", machine_type, tile_pos.0);

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
        Machine::PickerUpper => {
            commands
                .entity(machine)
                .insert(PlacedPickerUpperBundle::new(tile_pos, IVec2::X));
        }
    }
}

/// Pickup a machine, dropping and contained items on the ground
pub fn pickup_machine(
    player: Single<Entity, With<Player>>,
    targetted_machine: Single<
        (Entity, &Machine, Option<&Transporting>, &TilePos),
        (With<Placed>, With<Machine>, With<Targetted>),
    >,
    mut machine_lut: ResMut<MachineLUT>,
    mut commands: Commands,
) {
    let (machine, machine_type, items, pos) = *targetted_machine;
    info!("Picking up {:?} at {:?}", machine_type, pos.0);

    // Drop items out of machine
    if let Some(items) = items {
        for entity in items.iter() {
            commands
                .entity(entity)
                .remove::<TransportedItemBundle>()
                .insert(GroundItemBundle::new(&pos.as_world_pos()));
        }
    }

    // Move machine from ground to player
    match machine_type {
        Machine::Harvester => {
            commands.entity(machine).remove::<PlacedHarvesterBundle>();
        }
        Machine::Transporter => {
            commands.entity(machine).remove::<PlacedTransporterBundle>();
        }
        Machine::PickerUpper => {
            commands.entity(machine).remove::<PlacedPickerUpperBundle>();
        }
    }

    commands
        .entity(machine)
        .insert(HeldItemBundle::new(*player));

    // Remove LUT entry for the machine
    machine_lut.0.remove(pos);
}

/// Move items along the transporter
pub fn tick_transporters(
    mut transported_items: Query<(Entity, &mut Transform, &mut MachineState), With<TransportedBy>>,
    transporters: Query<(&MachineSpeed, &Direction, &Children, &TilePos), With<Transporter>>,
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

            // Items always travel along +X, as rotation is handled by machine-level transform
            transform.translation = Vec3::new(progress - 0.5, 0., transform.translation.z);

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
            &MachineState,
            &MachineSpeed,
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

/// Rotate a machine clockwise
pub fn rotate_machine(
    mut targetted_machine: Single<
        (&mut Direction, &mut Transform),
        (With<Placed>, With<Machine>, With<Targetted>),
    >,
) {
    info!("Rotating machine");

    // 90 degree turn clockwise
    let right_turn = IVec2::new(0, -1);

    targetted_machine.0.0 = right_turn.rotate(targetted_machine.0.0);
    targetted_machine.1.rotate_z(-FRAC_PI_2);
}

/// Advance the state of the picker-upper if there's an item on its tile
pub fn tick_pickerupper(
    machines: Query<(&TilePos, &mut MachineState, &MachineSpeed, &Direction), With<PickerUpper>>,
    machine_lut: Res<MachineLUT>,
    transporters: Query<(), With<Transporter>>,
    ground_items: Query<(Entity, &WorldPos), With<GroundItem>>,
    timer: Res<Time>,
    mut commands: Commands,
) {
    // Create LUT for ground items
    // TODO: replace this with some spatial query data structure stored as a resource
    let ground_items = ground_items.iter().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut hm, (entity, world_pos)| {
            // NOTE: +0.5 so we search centre of tile instead of origin corner
            hm.entry((world_pos + Vec2::splat(0.5)).tile())
                .or_default()
                .push(entity);

            hm
        },
    );

    for (machine_pos, mut state, speed, direction) in machines {
        let Some(items) = ground_items.get(machine_pos) else {
            // No items, reset progress
            state.0 = 0.;
            continue;
        };

        // Advance state
        state.0 += timer.delta_secs();
        if state.0 >= speed.0 {
            state.0 -= speed.0;

            // Pick up an item
            let item = *items
                .first()
                .expect("If hashmap has entry, there should be at least 1 item");
            commands.entity(item).remove::<GroundItemBundle>();

            let behind = machine_pos + direction.0;

            // Check if there's something beside it
            if let Some(adjacent_machine) = machine_lut.0.get(&behind) {
                // Put item in the machine next to it
                // TODO: Some abstraction over "machine that can accept items"
                if transporters.contains(*adjacent_machine) {
                    info!("Transferring item Picker-upper -> transporter");
                    // Another transporter
                    commands
                        .entity(item)
                        .insert(TransportedItemBundle::new(*adjacent_machine, direction));
                } else {
                    // Different type of machine
                    todo!()
                }
            } else {
                info!("Transferring item Picker-upper -> ground");
                // Drop item on ground
                commands
                    .entity(item)
                    .insert(GroundItemBundle::new(&behind.as_world_pos()));
            }
        }
    }
}
