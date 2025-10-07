use bevy::ecs::component::Component;

/// Indexes into terrain_sprites.png
#[derive(Component)]
#[repr(usize)]
pub enum TerrainSprite {
    Grass,
    Log,
    Bush,
    Water,
    Blank,
}
