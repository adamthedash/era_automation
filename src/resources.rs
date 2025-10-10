use bevy::{platform::collections::HashMap, prelude::*};
use rand::random_bool;

use crate::{
    consts::{CHUNK_SIZE, RESOURCE_DENSITY_BUSH, RESOURCE_DENSITY_LOG},
    map::{ChunkCreated, TilePos},
    player::Targettable,
    sprites::{ResourceSprite, SpriteSheet},
    utils,
};

pub struct ResourcePlugin;
impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceNodes>()
            .add_observer(spawn_resources);
    }
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ResourceType {
    Wood,
    Food,
    Water,
}

#[derive(Resource, Default)]
struct ResourceNodes(HashMap<TilePos, Entity>);

#[derive(Component)]
pub struct ResourceMarker;
/// Populate a chunk with naturally spawning resources
fn spawn_resources(
    event: On<ChunkCreated>,
    mut commands: Commands,
    mut resources: ResMut<ResourceNodes>,
    sprite_sheet: Res<SpriteSheet>,
) {
    let choices = [
        (ResourceSprite::Log, ResourceType::Wood),
        (ResourceSprite::Bush, ResourceType::Food),
    ];
    let weights = [RESOURCE_DENSITY_LOG, RESOURCE_DENSITY_BUSH];
    let total_weight = weights.iter().sum::<f32>().min(1.) as f64;

    let chunk_tile_pos = event.0.as_tile_pos();

    info!(
        "Spawning resources for chunk: {:?}, pos {:?}",
        event.0.0, chunk_tile_pos.0
    );
    for y in 0..CHUNK_SIZE.y {
        for x in 0..CHUNK_SIZE.x {
            if random_bool(total_weight) {
                let tile_pos = TilePos(chunk_tile_pos.0 + IVec2::new(x as i32, y as i32));

                let (sprite, res_type) = *utils::rand::choice(&choices, &weights);

                let entity = commands.spawn((
                    tile_pos,
                    Sprite {
                        image: sprite_sheet.image.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: sprite_sheet.layout.clone(),
                            index: sprite as usize,
                        }),
                        ..Default::default()
                    },
                    res_type,
                    // Z == 1 for resources
                    tile_pos.as_transform(1.),
                    ResourceMarker,
                    Targettable,
                ));

                resources.0.insert(tile_pos, entity.id());
            }
        }
    }
}
