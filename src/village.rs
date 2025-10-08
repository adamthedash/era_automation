use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        component::Component,
        query::Changed,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res},
    },
    platform::collections::HashMap,
    text::TextSpan,
    time::Time,
    ui::{FlexDirection, Node, percent, widget::Text},
};

pub struct VillagePlugin;
impl Plugin for VillagePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, (setup_village, setup_resource_display).chain())
            .add_systems(Update, (update_resources, update_resource_display));
    }
}

#[derive(Component)]
pub struct ResourceStockpile(f32);

/// Resource drain rate per second
#[derive(Component)]
pub struct ResourceDrainRate(f32);

#[derive(Component)]
pub struct ResourceName(String);

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ResourceType {
    Wood,
    Food,
    Water,
}

pub fn setup_village(mut commands: Commands) {
    commands.spawn((
        ResourceName("Wood".to_string()),
        ResourceDrainRate(1.),
        ResourceStockpile(100.),
        ResourceType::Wood,
    ));
    commands.spawn((
        ResourceName("Food".to_string()),
        ResourceDrainRate(1.),
        ResourceStockpile(0.),
        ResourceType::Food,
    ));
    commands.spawn((
        ResourceName("Water".to_string()),
        ResourceDrainRate(1.),
        ResourceStockpile(0.),
        ResourceType::Water,
    ));
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
