use crate::{
    map::{Chunks, TerrainData},
    resources::ResourceNodes,
    village::Stockpiles,
    weather::Wind,
};
use std::f32::consts::FRAC_PI_2;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    ground_items::{GroundItem, GroundItemBundle},
    items::ItemType,
    map::{TilePos, WorldPos},
    player::{HeldBy, HeldItemBundle, Holding, Player, TargettedBy},
    resources::{ResourceMarker, ResourceNodeLUT, ResourceNodeType},
    sprites::{GetSprite, SpriteSheets},
    village::{DepositEvent, ResourceStockpile, VillageCentre},
};

use super::bundles::*;
use super::components::*;

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
    machine_type.place(&mut commands.entity(machine), tile_pos, IVec2::X);
}

/// Pickup a machine, dropping and contained items on the ground
pub fn pickup_machine(
    player: Single<Entity, With<Player>>,
    targetted_machine: Single<
        (Entity, &Machine, Option<&Transporting>, &TilePos),
        (
            With<Placed>,
            With<Machine>,
            With<TargettedBy>,
            Without<VillageCentre>,
        ),
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
    machine_type.unplace(&mut commands.entity(machine));

    commands
        .entity(machine)
        .insert(HeldItemBundle::new(*player));

    // Remove LUT entry for the machine
    machine_lut.0.remove(pos);
}

/// Cycle through the sprites as the machine makes progress
pub fn animate_machine(
    machines: Query<(Entity, &MachineState, &Children, &AnimationSprites), With<Machine>>,
    sprite_entities: Query<(), With<Sprite>>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (machine, state, children, sprites) in machines {
        // Use the fractional part of the machine's progress as the animation phase.
        let sprite_index = (state.0.fract() * sprites.0.len() as f32) as usize;

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
        (With<Placed>, With<Machine>, With<TargettedBy>),
    >,
) {
    info!("Rotating machine");

    // 90 degree turn clockwise
    let right_turn = IVec2::new(0, -1);

    targetted_machine.0.0 = right_turn.rotate(targetted_machine.0.0);
    targetted_machine.1.rotate_z(-FRAC_PI_2);
}

/// Transfer items from the ether into machines
pub fn transfer_items(
    mut reader: MessageReader<TransferItem>,
    machines: Query<(EntityRef, &Machine, &AcceptsItems), With<Placed>>,
    items: Query<&ItemType>,
    mut stockpiles: Stockpiles<&mut ResourceStockpile, (Without<ItemType>, Without<Machine>)>,
    mut commands: Commands,
) {
    for TransferItem {
        item,
        target_machine,
    } in reader.read()
    {
        // Get target machine
        let (machine, machine_type, accceptable_items) = machines
            .get(*target_machine)
            .expect("Target machine does not exist!");

        // Verify that this transfer can happen (this should already be checked before the transfer
        // request, so this is a sanity check)
        let item_type = items.get(*item).expect("Item doesn't exist!");
        assert!(
            accceptable_items.can_accept(item_type),
            "Machine cannot accept this item"
        );

        use Machine::*;
        match machine_type {
            Transporter => {
                let direction = machine
                    .get::<Direction>()
                    .expect("Machine does not have a direction!");

                commands
                    .entity(*item)
                    .insert(TransportedItemBundle::new(machine.id(), direction));
            }
            VillageCentre => {
                let resource = item_type
                    .resource_type()
                    .expect("Item does not provide a resource!");

                let mut stockpile = stockpiles
                    .get_mut(&resource)
                    .expect("Stockpile not created!");

                // TODO: Different items giving different amounts of a resource
                let amount = 1;
                stockpile.0 += amount as f32;

                // Remove the item
                commands.entity(*item).despawn();

                commands.trigger(DepositEvent { resource, amount });
            }
            _ => unreachable!("Machine accepts items but logic not here!"),
        };
    }
}

/// Advance the state of the harvesters if there is a resource beside it
pub fn tick_resource_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &HarvestableNodes,
        ),
        With<Harvester>,
    >,
    resources: ResourceNodes<(&ResourceNodeType, &ItemType), With<ResourceMarker>>,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_producers: Machines<&CurrentEnergy, With<Placed>>,
    timer: Res<Time>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (tile_pos, mut state, speed, power, direction, harvestable_nodes) in harvesters {
        // Check if there's a harvestable node in front of the machine
        let resource_pos = tile_pos + direction.0;

        let Some((resource_type, item_type)) = resources.get(&resource_pos) else {
            // No resource, so reset progress
            state.0 = 0.;
            continue;
        };

        // Check that resource can be harvested by this machine
        if !harvestable_nodes.0.contains(resource_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Accumulate energy produced by adjacent machines
        let energy_supply = tile_pos
            .adjacent()
            .flat_map(|pos| energy_producers.get(&pos).map(|e| e.0))
            .sum::<f32>();

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.);
        let work_rate = speed.0 * satisfaction;

        // Advance progress (1.0 == one completed action)
        state.0 += work_rate * timer.delta_secs();

        // Check if harvest has been completed
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Spawn an item
        let output_pos = tile_pos - direction.0;
        let item = commands.spawn(*item_type).id();
        item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&output_pos)
            && acceptable_items.can_accept(item_type)
        {
            info!("Transferring item Harvester -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Harvester -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&output_pos.as_world_pos()));
        }
    }
}

/// Advance the state of the terrain harvesters
pub fn tick_terrain_harvesters(
    harvesters: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &HarvestableTerrain,
        ),
        With<Harvester>,
    >,
    chunks: Chunks<&TerrainData>,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_producers: Machines<&CurrentEnergy, With<Placed>>,
    timer: Res<Time>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (tile_pos, mut state, speed, power, direction, harvestable_terrain) in harvesters {
        // Check if there's a harvestable node under of the machine
        let resource_pos = tile_pos + direction.0;

        // Get terrain under the machine
        let (chunk_pos, offset) = resource_pos.to_chunk_offset();
        let terrain_data = chunks.get(&chunk_pos).expect("Chunk data not generated");
        let terrain_type = &terrain_data.0[offset.y as usize][offset.x as usize];

        // Check that resource can be harvested by this machine
        if !harvestable_terrain.0.contains(terrain_type) {
            // Can't harvest this type of node, so reset progress
            state.0 = 0.;
            continue;
        }

        // Check that the terrain can produce something
        let Some(item_type) = terrain_type.item_type() else {
            // No item given
            state.0 = 0.;
            continue;
        };

        // Accumulate energy produced by adjacent machines (e.g., windmills)
        let energy_supply = tile_pos
            .adjacent()
            .flat_map(|pos| energy_producers.get(&pos).map(|e| e.0))
            .sum::<f32>();

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        // Tick the machine (scaled by available adjacent energy)
        state.0 += work_rate * timer.delta_secs();

        // Check if harvest has been completed
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Spawn an item
        let output_pos = tile_pos + direction.0;
        let item = commands.spawn(item_type).id();
        item_type.spawn_sprite(&mut commands, &sprite_sheets, Some(item));

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&output_pos)
            && acceptable_items.can_accept(&item_type)
        {
            info!("Transferring item Harvester -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Harvester -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&output_pos.as_world_pos()));
        }
    }
}

/// Move items along the transporter
pub fn tick_transporters(
    mut transported_items: Query<
        (Entity, &mut Transform, &mut MachineState, &ItemType),
        With<TransportedBy>,
    >,
    transporters: Query<
        (
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
            &Children,
            &TilePos,
        ),
        With<Transporter>,
    >,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    energy_producers: Machines<&CurrentEnergy, With<Placed>>,
    timer: Res<Time>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    for (speed, power, direction, children, machine_pos) in transporters {
        // Compute available adjacent energy for this transporter
        let energy_supply = machine_pos
            .adjacent()
            .flat_map(|pos| energy_producers.get(&pos).map(|e| e.0))
            .sum::<f32>();

        // Calculate work rate based on current power supply
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        for child in children {
            let Ok((item, mut transform, mut progress, item_type)) =
                transported_items.get_mut(*child)
            else {
                // Non-item child
                continue;
            };

            // Move item along the current transporter (state.0 counts actions completed)
            progress.0 += work_rate * timer.delta_secs();

            // Items always travel along +X, as rotation is handled by machine-level transform
            transform.translation = Vec3::new(progress.0 - 0.5, 0., transform.translation.z);

            // Check if the item has gone off the end
            if progress.0 < 1. {
                // Still on the belt
                continue;
            }

            // Move item off the conveyor
            let adjacent_pos = machine_pos + direction.0;
            if let Some((machine, machine_type, acceptable_items)) = machines.get(&adjacent_pos)
                && acceptable_items.can_accept(item_type)
            {
                info!("Transferring item Transporter -> {:?}", machine_type);

                // Request to transfer to the target machine
                transfer_items.write(TransferItem {
                    item,
                    target_machine: machine,
                });
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

/// Advance the state of the picker-upper if there's an item on its tile
pub fn tick_pickerupper(
    picker_uppers: Query<
        (
            &TilePos,
            &mut MachineState,
            &MachineSpeed,
            &PowerConsumption,
            &Direction,
        ),
        With<PickerUpper>,
    >,
    machines: Machines<(Entity, &Machine, &AcceptsItems), With<Placed>>,
    ground_items: Query<(Entity, &WorldPos, &ItemType), With<GroundItem>>,
    energy_producers: Machines<&CurrentEnergy, With<Placed>>,
    timer: Res<Time>,
    mut commands: Commands,
    mut transfer_items: MessageWriter<TransferItem>,
) {
    // Create LUT for ground items
    // TODO: replace this with some spatial query data structure stored as a resource
    let ground_items = ground_items.iter().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut hm, (entity, world_pos, item_type)| {
            // NOTE: +0.5 so we search centre of tile instead of origin corner
            hm.entry((world_pos + Vec2::splat(0.5)).tile())
                .or_default()
                .push((entity, item_type));

            hm
        },
    );

    for (machine_pos, mut state, speed, power, direction) in picker_uppers {
        let Some(items) = ground_items.get(machine_pos) else {
            // No items, reset progress
            state.0 = 0.;
            continue;
        };

        // Accumulate energy produced by adjacent machines
        let energy_supply = machine_pos
            .adjacent()
            .flat_map(|pos| energy_producers.get(&pos).map(|e| e.0))
            .sum::<f32>();

        // Calculate work rate at current power level
        let satisfaction = (energy_supply / power.0).min(1.0);
        let work_rate = speed.0 * satisfaction;

        // Advance state
        state.0 += work_rate * timer.delta_secs();
        if state.0 < 1.0 {
            // Not done yet
            continue;
        }
        state.0 -= 1.0;

        // Pick up an item
        let (item, item_type) = *items
            .first()
            .expect("If hashmap has entry, there should be at least 1 item");
        commands.entity(item).remove::<GroundItemBundle>();

        let behind = machine_pos + direction.0;

        // Check if there's something beside it
        if let Some((machine, machine_type, acceptable_items)) = machines.get(&behind)
            && acceptable_items.can_accept(item_type)
        {
            info!("Transferring item Picker-upper -> {:?}", machine_type);

            // Request to transfer to the target machine
            transfer_items.write(TransferItem {
                item,
                target_machine: machine,
            });
        } else {
            info!("Transferring item Picker-upper -> ground");
            // Drop item on ground
            commands
                .entity(item)
                .insert(GroundItemBundle::new(&behind.as_world_pos()));
        }
    }
}

/// Tick all placed windmills and update their current production in the `CurrentEnergy` component.
pub fn tick_windmills(
    wind: Res<Wind>,
    timer: Res<Time>,
    windmills: Query<
        (
            &Direction,
            &mut CurrentEnergy,
            &MachineSpeed,
            &mut MachineState,
        ),
        With<Windmill>,
    >,
) {
    for (direction, mut current_energy, speed, mut state) in windmills {
        // Compute alignment in [-1, 1]; only positive alignment produces energy.
        let alignment = direction.0.as_vec2().dot(wind.direction_vec()).max(0.0);

        // Update energy production rate
        current_energy.0 = wind.speed * alignment;

        // Update animation
        let produced = current_energy.0 * timer.delta_secs();
        state.0 = (state.0 + produced) % speed.0;
    }
}
