use bevy::prelude::*;

use crate::player::HeldItem;

pub fn key_just_pressed(key: KeyCode) -> impl FnMut(Res<ButtonInput<KeyCode>>) -> bool {
    move |inputs| inputs.just_pressed(key)
}

pub fn empty_hands(held_item: Query<(), With<HeldItem>>) -> bool {
    held_item.is_empty()
}
