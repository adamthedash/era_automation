use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{bundles::*, components::*};
pub use super::{
    harvester::systems::*, network::systems::*, picker_upper::systems::*, transporter::systems::*,
    windmill::systems::*,
};
use crate::{
    ground_items::GroundItemBundle,
    items::ItemType,
    map::{TilePos, WorldPos},
    player::{HeldBy, HeldItemBundle, Holding, Player, TargettedBy},
    resources::ResourceNodeLUT,
    sprites::{GetSprite, SpriteSheets},
    village::{DepositEvent, ResourceStockpile, Stockpiles, VillageCentre},
};

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
