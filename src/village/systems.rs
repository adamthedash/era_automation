use bevy::{platform::collections::HashMap, prelude::*};

use super::components::*;
use crate::{
    consts::Z_RESOURCES,
    container::{ContainedBundle, ContainedBy, Container, Contains},
    items::ItemType,
    machines::{AcceptsItems, Machine, MachineLUT, Placed, TransferItem},
    map::TilePos,
    player::{HeldBy, HeldItemBundle, Targettable, TargettedBy},
    resources::ResourceType,
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

/// Initialise the starting resource stockpiles
pub fn setup_stockpiles(mut commands: Commands, mut lut: ResMut<StockpileLUT>) {
    use ResourceType::*;
    for res_type in [Wood, Food, Water] {
        let entity = commands.spawn((
            res_type,
            ResourceName(format!("{res_type:?}")),
            ResourceDrainRate(1. / 6.),
            ResourceStockpile(100.),
        ));

        lut.0.insert(res_type, entity.id());
    }
}

/// Regular ticking of resources
pub fn update_resources(
    resources: Query<(&mut ResourceStockpile, &ResourceDrainRate)>,
    time: Res<Time>,
) {
    for (mut stock, drain_rate) in resources {
        stock.0 -= drain_rate.0 * time.delta_secs();
        stock.0 = stock.0.max(0.);
    }
}

/// Create UI elements to display resources
pub fn setup_resource_display(
    mut commands: Commands,
    query: Query<(&ResourceName, &ResourceStockpile, &ResourceType)>,
) {
    commands
        .spawn(Node {
            // Root of resource panel
            flex_direction: FlexDirection::Column,
            height: percent(100),
            ..Default::default()
        })
        .with_children(|root| {
            for (name, stock, res_type) in query {
                // Individual resource nodes
                root.spawn((Node {
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("{}: {:.0}", name.0, stock.0.ceil())),
                            *res_type,
                        ));
                    });
            }
        });
}

/// Sync UI with underlying data
pub fn update_resource_display(
    resources: Query<
        (&ResourceName, &ResourceStockpile, &ResourceType),
        Changed<ResourceStockpile>,
    >,
    displays: Query<(&mut Text, &ResourceType)>,
) {
    // Create new text entries for changed resources
    let mut new_texts = resources
        .iter()
        .map(|(name, stock, res_type)| (*res_type, format!("{}: {:.0}", name.0, stock.0.ceil())))
        .collect::<HashMap<_, _>>();

    for (mut text, res_type) in displays {
        if let Some(new_text) = new_texts.remove(res_type) {
            text.0 = new_text
        }
    }
}

/// Spawn the village centre that's used to deposit items
pub fn spawn_village_centre(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    mut machine_lut: ResMut<MachineLUT>,
) {
    let pos = TilePos(IVec2::ZERO);
    let village = commands
        .spawn((
            pos,
            pos.as_transform(Z_RESOURCES),
            VillageCentre,
            Targettable,
            Machine::VillageCentre,
            AcceptsItems::Whitelist(vec![ItemType::Berry, ItemType::Log, ItemType::Water]),
            Placed,
        ))
        .id();

    ResourceSprite::House.spawn_sprite(&mut commands, &sprite_sheets, Some(village));

    machine_lut.0.insert(pos, village);
}

/// Deposit a held item into the village
pub fn deposit_resource(
    villages: Query<(Entity, &AcceptsItems, &TargettedBy), With<VillageCentre>>,
    held_items: Query<(Entity, &ItemType, &HeldBy), Without<Container>>,
    held_containers: Query<(&Contains, &HeldBy)>,
    contained_items: Query<&ItemType, With<ContainedBy>>,
    mut commands: Commands,
    mut writer: MessageWriter<TransferItem>,
) {
    // For each village centre, deposit items held by the entity that is targeting that centre
    for (village_entity, accepts, targetted_by) in villages.iter() {
        let targetter = targetted_by.0;

        // Move held depositables to the void (only those held by the targetter)
        let held = held_items
            .iter()
            .filter_map(|(item, item_type, held_by)| {
                (held_by.0 == targetter && accepts.can_accept(item_type)).then_some(item)
            })
            .inspect(|item| {
                commands.entity(*item).remove::<HeldItemBundle>();
            })
            .collect::<Vec<_>>();

        // Move depositables from containers held by the targetter to the void
        let contained = held_containers
            .iter()
            .filter(|(_, held_by)| held_by.0 == targetter)
            .flat_map(|(children, _)| {
                children.iter().filter(|item| {
                    let item_type = contained_items
                        .get(*item)
                        .expect("Following contained relationship, so this should exist");

                    accepts.can_accept(item_type)
                })
            })
            .inspect(|item| {
                commands.entity(*item).remove::<ContainedBundle>();
            });

        // Trigger transfer to village
        held.into_iter().chain(contained).for_each(|item| {
            writer.write(TransferItem {
                item,
                target_machine: village_entity,
            });
        });
    }
}
