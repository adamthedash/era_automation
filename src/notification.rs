use std::time::Duration;

use bevy::prelude::*;

use crate::knowledge::UnlockEvent;

pub struct NotificationPlugin;
impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_notification_system)
            .add_systems(Update, update_notifications)
            .add_observer(unlock_notification);
    }
}

/// Spawn the noficiation box
#[derive(Component)]
struct NotificationBox;
fn init_notification_system(mut commands: Commands) {
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            position_type: PositionType::Absolute,
            bottom: px(10),
            left: px(10),
            ..Default::default()
        },
        NotificationBox,
    ));
}

/// How long left for a notification to display
#[derive(Component)]
struct DisplayDuration(Duration);

/// Spawns a notification for a knowledge unlock
fn unlock_notification(
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

/// Remove notifications when their timer has expired
fn update_notifications(
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
