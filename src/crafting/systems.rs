use bevy::prelude::*;

use super::components::*;

use crate::{
    consts::PLAYER_REACH,
    knowledge::Unlocked,
    map::{TilePos, WorldPos},
    player::{HeldItemBundle, Holding, Player},
    sprites::{GetSprite, SpriteSheets},
    village::{ResourceStockpile, StockpileLut, VillageCentre},
};

/// Checks if the player is within range of a crafting station
pub fn check_near_crafting_station(
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

/// Shows the recipes that can be crafted
pub fn show_crafting_ui(
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

/// Interaction with crafting buttons
pub fn crafting_button(
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

/// Process crafting requests
pub fn try_craft_recipes(
    mut reader: MessageReader<CraftRecipe>,
    stockpile_lut: Res<StockpileLut>,
    mut resources: Query<&mut ResourceStockpile>,
    player: Single<(Entity, Has<Holding>), (With<Player>, Without<ResourceStockpile>)>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    let (player, held_item) = *player;

    // TODO: Might need to just process 1 per frame here
    for CraftRecipe(recipe) in reader.read() {
        info!("Attempting to craft: {:?}", recipe.product);

        if held_item {
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
            let mut entity_commands = commands.spawn((HeldItemBundle::new(player), recipe.product));
            recipe.product.add_extra_components(&mut entity_commands);
            let entity = entity_commands.id();

            recipe
                .product
                .spawn_sprite(&mut commands, &sprite_sheets, Some(entity));
        } else {
            // Not enough resources
            commands.trigger(FailedCraft {
                recipe: recipe.clone(),
                reason: FailedCraftReason::NotEnoughResources,
            });
        }
    }
}
