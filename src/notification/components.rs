use std::time::Duration;

use bevy::prelude::*;

/// Top-level marker for the notification box UI element
#[derive(Component)]
pub struct NotificationBox;

/// How long left for a notification to display
#[derive(Component)]
pub struct DisplayDuration(pub Duration);
