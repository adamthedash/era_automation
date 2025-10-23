use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use era_automation::{
    consts::CHUNK_LOAD_RADIUS,
    map::{
        ChunkLUT, ChunkPos, CreateChunk, WorldGenerator, WorldPos,
        systems::{create_chunks, init_world_gen, update_transforms},
    },
    player::{
        Player,
        systems::{move_player, setup_player},
    },
    sprites::SpritePlugin,
    utils::noise::perlin_stack,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpritePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EguiPlugin::default())
        .add_systems(Update, change_terrain_gen)
        .add_systems(EguiPrimaryContextPass, ui_config)
        .add_systems(Update, zoom_camera)
        .init_resource::<WorldGenConfig>()
        .run();
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_world_gen)
            .add_systems(
                Update,
                (
                    (despawn_chunks, spawn_chunks, create_chunks).chain(),
                    update_transforms,
                ),
            )
            .init_resource::<ChunkLUT>()
            .add_message::<CreateChunk>();
    }
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

/// Delete exisitng chunks so they can be regenerated
fn despawn_chunks(
    world_gen: Res<WorldGenerator>,
    mut chunk_lut: ResMut<ChunkLUT>,
    mut commands: Commands,
) {
    if !world_gen.is_changed() {
        return;
    }

    for (_, entity) in chunk_lut.0.drain() {
        commands.entity(entity).despawn();
    }
}

#[derive(Resource, PartialEq, Clone, Debug)]
struct WorldGenConfig {
    seed: u64,
    num_octaves: usize,
    amplitude: f64,
    persistence: f64,
    scale: f64,
    offset: f64,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            num_octaves: 4,
            amplitude: 0.5,
            persistence: 0.5,
            scale: 1. / 16.,
            offset: 0.,
        }
    }
}

/// Propogate config changes to the world generator
fn change_terrain_gen(mut world_gen: ResMut<WorldGenerator>, config: Res<WorldGenConfig>) {
    if !config.is_changed() {
        return;
    }

    world_gen.height = Box::new(perlin_stack(
        config.seed,
        config.num_octaves,
        config.amplitude,
        config.persistence,
        config.scale,
        config.offset,
    ));
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, move_player);
    }
}

fn ui_config(
    mut ctx: EguiContexts,
    mut local_config: Local<WorldGenConfig>,
    mut global_config: ResMut<WorldGenConfig>,
) -> Result {
    egui::Window::new("World Gen").show(ctx.ctx_mut()?, |ui| {
        ui.label("Seed");
        ui.add(egui::DragValue::new(&mut local_config.seed));

        ui.label("Octaves");
        ui.add(egui::DragValue::new(&mut local_config.num_octaves).range(1..=128));

        ui.label("Amplitude");
        let exp = 1.;
        let mut old = local_config.amplitude.powf(exp);
        ui.add(egui::DragValue::new(&mut old).range(0.01..=1.).speed(0.01));
        local_config.amplitude = old.powf(1. / exp);

        ui.label("Persistence");
        let exp = 1.;
        let mut old = local_config.persistence.powf(exp);
        ui.add(egui::DragValue::new(&mut old).range(0.01..=1.).speed(0.01));
        local_config.persistence = old.powf(1. / exp);

        ui.label("Scale 2^x");
        let mut old = local_config.scale.log2();
        ui.add(egui::DragValue::new(&mut old).speed(0.1));
        local_config.scale = old.exp2();

        ui.label("Offset");
        ui.add(egui::DragValue::new(&mut local_config.offset).speed(0.1));
    });

    if *global_config != *local_config {
        *global_config = local_config.clone();
    }

    Ok(())
}

fn zoom_camera(
    camera: Query<&mut Transform, With<Camera2d>>,
    mouse: Res<AccumulatedMouseScroll>,
    timer: Res<Time>,
) {
    if mouse.delta.y == 0. {
        return;
    }

    for mut transform in camera {
        let delta = mouse.delta.y * timer.delta_secs();

        transform.scale *= Vec2::splat(1. + delta.tanh()).extend(1.);
    }
}
