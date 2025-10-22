use bevy::prelude::*;

use crate::player::{Holding, Player};

pub fn key_just_pressed(key: KeyCode) -> impl FnMut(Res<ButtonInput<KeyCode>>) -> bool {
    move |inputs| inputs.just_pressed(key)
}

pub fn empty_hands(held_item: Single<Has<Holding>, With<Player>>) -> bool {
    !(*held_item)
}
