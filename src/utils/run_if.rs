use bevy::prelude::*;

pub fn key_just_pressed(key: KeyCode) -> impl FnMut(Res<ButtonInput<KeyCode>>) -> bool {
    move |inputs| inputs.just_pressed(key)
}
