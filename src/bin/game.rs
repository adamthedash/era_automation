use bevy::prelude::*;
use era_automation::{
    consts::CHUNK_LOAD_RADIUS,
    container::ContainerPlugin,
    crafting::CraftingPlugin,
    debug::DebugPlugin,
    ground_items::GroundItemPlugin,
    knowledge::KnowledgePlugin,
    machines::MachinePlugin,
    map::{ChunkLUT, ChunkPos, CreateChunk, MapPlugin, WorldPos},
    notification::NotificationPlugin,
    player::{Player, PlayerPlugin},
    resources::ResourcePlugin,
    sprites::SpritePlugin,
    village::VillagePlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpritePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(VillagePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ResourcePlugin)
        .add_plugins(KnowledgePlugin)
        .add_plugins(NotificationPlugin)
        .add_plugins(CraftingPlugin)
        .add_plugins(GroundItemPlugin)
        .add_plugins(ContainerPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(MachinePlugin)
        .add_systems(Update, spawn_chunks)
        .run();
}

/// Spawn chunks around the player if they're not generated yet
fn spawn_chunks(
    player: Single<&WorldPos, With<Player>>,
    mut messages: MessageWriter<CreateChunk>,
    chunk_lut: Res<ChunkLUT>,
) {
    let player_chunk = player.chunk();

    for x in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
        for y in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
            let chunk_pos = ChunkPos(IVec2::new(player_chunk.0.x + x, player_chunk.0.y + y));
            if !chunk_lut.0.contains_key(&chunk_pos) {
                messages.write(CreateChunk(chunk_pos));
            }
        }
    }
}
