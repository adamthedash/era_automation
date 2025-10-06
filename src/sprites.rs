use bevy::ecs::component::Component;

#[derive(Component)]
pub enum TerrainSprite {
    Grass,
    Log,
    Bush,
    Water,
    Blank,
}
