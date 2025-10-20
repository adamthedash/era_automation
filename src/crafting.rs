use bevy::prelude::*;

use crate::{
    consts::PLAYER_REACH,
    items::ItemType,
    knowledge::Unlocked,
    map::{TilePos, WorldPos},
    player::{HeldItem, Player, held_item_bundle},
    resources::ResourceType,
    sprites::{GetSprite, SpriteSheets},
    village::{ResourceStockpile, StockpileLut, VillageCentre},
};

pub struct CraftingPlugin;
impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CraftRecipe>().add_systems(
            Update,
            (
                check_near_crafting_station,
                show_crafting_ui,
                crafting_button,
                try_craft_recipes,
            ),
        );
    }
}

#[derive(Component)]
struct NearCraftingStation;

/// Checks if the player is within range of a crafting station
fn check_near_crafting_station(
    player: Single<(Entity, &WorldPos), With<Player>>,
    village: Query<&TilePos, With<VillageCentre>>,
    mut commands: Commands,
) {
    let (entity, player_pos) = *player;

    let near_station = village.iter().any(|tile_pos| {
        tile_pos.as_world_pos().0.distance_squared(player_pos.0) <= PLAYER_REACH.powi(2)
    });

    if near_station {
        commands.entity(entity).insert_if_new(NearCraftingStation);
    } else {
        commands.entity(entity).remove::<NearCraftingStation>();
    }
}

/// Resource requirements to craft an item
#[derive(Component, Clone, Debug)]
pub struct Recipe {
    pub reqs: Vec<(ResourceType, usize)>,
    pub product: ItemType,
}

#[derive(Component)]
struct CraftingWindow;

#[derive(Component)]
struct CraftingNode;

/// Shows the recipes that can be crafted
fn show_crafting_ui(
    near_station: Single<Option<&NearCraftingStation>, With<Player>>,
    crafting_window: Option<Single<Entity, With<CraftingWindow>>>,
    recipes: Query<&Recipe, With<Unlocked>>,
    mut commands: Commands,
) {
    if near_station.is_some() {
        if crafting_window.is_none() {
            // Show window
            commands
                .spawn((
                    CraftingWindow,
                    Node {
                        display: Display::Flex,
                        position_type: PositionType::Absolute,
                        bottom: px(10),
                        right: px(10),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    // Show each recipe
                    for recipe in recipes {
                        let text = format!(
                            "{:?} ({})",
                            recipe.product,
                            recipe
                                .reqs
                                .iter()
                                .map(|(res, amount)| { format!("{}x {:?}", amount, res) })
                                .collect::<Vec<_>>()
                                .join(", ")
                        );

                        parent.spawn((
                            Button,
                            CraftingNode,
                            recipe.clone(),
                            children![(Text(text),)],
                            // Render
                            Node {
                                border: UiRect::all(px(2)),
                                padding: UiRect::all(px(4)),
                                ..Default::default()
                            },
                            BorderColor::all(Color::WHITE),
                            BorderRadius::MAX,
                            BackgroundColor(Color::BLACK),
                        ));
                    }
                });
        }
    } else if let Some(entity) = crafting_window {
        // Despawn window
        commands.entity(*entity).despawn();
    }
}

#[derive(Message)]
pub struct CraftRecipe(pub Recipe);

/// Interaction with crafting buttons
fn crafting_button(
    buttons: Query<(&Interaction, &mut BackgroundColor, &Recipe), Changed<Interaction>>,
    mut writer: MessageWriter<CraftRecipe>,
) {
    for (interaction, mut bg_color, recipe) in buttons {
        match interaction {
            Interaction::Pressed => {
                writer.write(CraftRecipe(recipe.clone()));
            }
            Interaction::Hovered => *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            Interaction::None => *bg_color = BackgroundColor(Color::BLACK),
        }
    }
}

#[derive(Debug)]
pub enum FailedCraftReason {
    NotEnoughResources,
    HoldingItem,
}

#[derive(Event)]
pub struct FailedCraft {
    pub recipe: Recipe,
    pub reason: FailedCraftReason,
}

/// Process crafting requests
fn try_craft_recipes(
    mut reader: MessageReader<CraftRecipe>,
    stockpile_lut: Res<StockpileLut>,
    mut resources: Query<&mut ResourceStockpile>,
    player: Single<(Entity, &Transform), With<Player>>,
    held_item: Option<Single<(), With<HeldItem>>>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    let (player, player_transform) = *player;

    // TODO: Might need to just process 1 per frame here
    for CraftRecipe(recipe) in reader.read() {
        info!("Attempting to craft: {:?}", recipe.product);
        if held_item.is_some() {
            commands.trigger(FailedCraft {
                recipe: recipe.clone(),
                reason: FailedCraftReason::HoldingItem,
            });
            // Crafted item is given to the player, so they need to be able to hold it
            continue;
        }

        // Check if we've got enough resources
        if recipe.reqs.iter().all(|(res_type, amount)| {
            let stockpile = stockpile_lut
                .0
                .get(res_type)
                .and_then(|entity| resources.get(*entity).ok());

            stockpile.is_some_and(|stockpile| stockpile.0 >= *amount as f32)
        }) {
            // Enough resources, craft it!

            // Remove resources
            for (res_type, amount) in &recipe.reqs {
                let mut stockpile = stockpile_lut
                    .0
                    .get(res_type)
                    .and_then(|entity| resources.get_mut(*entity).ok())
                    .expect("Already checked they exist above");

                stockpile.0 -= *amount as f32;
            }

            // Give item to player
            commands.entity(player).with_child((
                recipe.product,
                held_item_bundle(player_transform),
                recipe.product.get_sprite(&sprite_sheets),
            ));
        } else {
            // Not enough resources
            commands.trigger(FailedCraft {
                recipe: recipe.clone(),
                reason: FailedCraftReason::NotEnoughResources,
            });
        }
    }
}
