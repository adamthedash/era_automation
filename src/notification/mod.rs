mod components;
mod systems;

use bevy::prelude::*;

pub use components::*;
use systems::*;

pub struct NotificationPlugin;
impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_notification_system)
            .add_systems(Update, update_notifications)
            .add_observer(failed_craft)
            .add_observer(unlock_notification);
    }
}
