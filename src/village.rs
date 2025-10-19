use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    consts::Z_RESOURCES,
    map::TilePos,
    player::{HeldItem, Targettable, Targetted},
    resources::{ResourceAmount, ResourceType},
    sprites::{ResourceSprite, SpriteSheets},
    utils::run_if::key_just_pressed,
};

pub struct VillagePlugin;
impl Plugin for VillagePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<StockpileLut>()
            .add_systems(Startup, (setup_village, setup_resource_display).chain())
            .add_systems(Startup, spawn_village_centre)
            .add_systems(
                Update,
                (
                    update_resources,
                    update_resource_display,
                    deposit_resource.run_if(key_just_pressed(KeyCode::Space)),
                ),
            );
    }
}

#[derive(Component)]
pub struct ResourceStockpile(pub f32);

/// Resource drain rate per second
#[derive(Component)]
pub struct ResourceDrainRate(f32);

#[derive(Component)]
pub struct ResourceName(String);

#[derive(Resource, Default)]
pub struct StockpileLut(pub HashMap<ResourceType, Entity>);

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

/// Marker for village building
#[derive(Component)]
pub struct VillageCentre;
/// Spawn the village centre that's used to deposit items
fn spawn_village_centre(mut commands: Commands, sprite_sheet: Res<SpriteSheets>) {
    let pos = TilePos(IVec2::ZERO);
    commands.spawn((
        pos,
        pos.as_transform(Z_RESOURCES),
        Sprite {
            image: sprite_sheet.resources.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_sheet.resources.layout.clone(),
                index: ResourceSprite::House as usize,
            }),
            ..Default::default()
        },
        VillageCentre,
        Targettable,
    ));
}

/// Deposit a held resource into the village
fn deposit_resource(
    mut commands: Commands,
    _village: If<Single<(), (With<VillageCentre>, With<Targetted>)>>,
    items: Query<(Entity, &ResourceType, &ResourceAmount), With<HeldItem>>,
    mut stockpiles: Query<(&mut ResourceStockpile, &ResourceType)>,
) {
    for (entity, res_type, amount) in items {
        let (mut stockpile, _) = stockpiles
            .iter_mut()
            .find(|(_, stock_type)| *stock_type == res_type)
            .expect("Stockpile not created");

        stockpile.0 += amount.0 as f32;

        // Remove the item
        commands.entity(entity).despawn();
    }
}
