use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    consts::Z_RESOURCES,
    items::ItemType,
    map::TilePos,
    player::{HeldBy, Targettable},
    resources::{ResourceAmount, ResourceType},
    sprites::{GetSprite, ResourceSprite, SpriteSheets},
};

use super::components::*;

/// Initialise the starting resource stockpiles
pub fn setup_village(mut commands: Commands, mut lut: ResMut<StockpileLut>) {
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
pub fn spawn_village_centre(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
    let pos = TilePos(IVec2::ZERO);
    let village = commands
        .spawn((
            pos,
            pos.as_transform(Z_RESOURCES),
            VillageCentre,
            Targettable,
        ))
        .id();

    ResourceSprite::House.spawn_sprite(&mut commands, &sprite_sheets, Some(village));
}

/// Deposit a held item into the village
pub fn deposit_resource(
    mut commands: Commands,
    items: Query<(Entity, &ItemType, &ResourceAmount), With<HeldBy>>,
    mut stockpiles: Query<(&mut ResourceStockpile, &ResourceType)>,
) {
    for (entity, item_type, amount) in items {
        let Some(resource) = item_type.resource_type() else {
            // No resource provided by this item
            continue;
        };

        let (mut stockpile, _) = stockpiles
            .iter_mut()
            .find(|(_, stock_type)| **stock_type == resource)
            .expect("Stockpile not created");

        stockpile.0 += amount.0 as f32;

        // Remove the item
        commands.entity(entity).despawn();

        commands.trigger(DepositEvent {
            resource,
            amount: amount.0,
        });
    }
}
