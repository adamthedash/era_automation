use std::time::Duration;

use bevy::prelude::*;

use super::components::*;
use crate::{crafting::FailedCraft, knowledge::UnlockEvent};

/// Spawn the notification box
pub fn init_notification_system(mut commands: Commands) {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::ColumnReverse,
            position_type: PositionType::Absolute,
            bottom: px(10),
            left: px(10),
            ..Default::default()
        },
        NotificationBox,
    ));
}

/// Remove notifications when their timer has expired
pub fn update_notifications(
    notifications: Query<(Entity, &mut DisplayDuration)>,
    timer: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut remaining) in notifications {
        if let Some(new_remaining) = remaining.0.checked_sub(timer.delta()) {
            // Still some time left
            remaining.0 = new_remaining;
        } else {
            // Time elapsed, despawn entity
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a notification for a knowledge unlock
pub fn unlock_notification(
    event: On<UnlockEvent>,
    display_box: Single<Entity, With<NotificationBox>>,
    mut commands: Commands,
) {
    commands.entity(*display_box).with_child((
        Text(format!("Unlocked knowledge: {}", event.name)),
        DisplayDuration(Duration::from_secs(5)),
        Node {
            position_type: PositionType::Relative,
            ..Default::default()
        },
    ));
}

/// Spawns a notification when a craft is failed
pub fn failed_craft(
    event: On<FailedCraft>,
    display_box: Single<Entity, With<NotificationBox>>,
    mut commands: Commands,
) {
    commands.entity(*display_box).with_child((
        Text(format!(
            "Couldn't craft {:?} {:?}",
            event.recipe.product, event.reason
        )),
        DisplayDuration(Duration::from_secs(5)),
        Node {
            position_type: PositionType::Relative,
            ..Default::default()
        },
    ));
}
