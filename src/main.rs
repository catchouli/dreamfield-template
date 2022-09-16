mod sim;

use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

use cgmath::{vec4, vec3, vec2, Vector2, Vector3, perspective, Deg, Matrix4, SquareMatrix, Matrix3};
use include_dir::{include_dir, Dir};

use dreamfield_system::GameHost;
use dreamfield_system::components::{EntityName, Transform};
use dreamfield_system::systems::entity_spawner::EntitySpawnRadius;
use dreamfield_system::world::WorldChunkManager;
use dreamfield_renderer::gl_backend::TextureParams;
use dreamfield_renderer::components::{PlayerCamera, Visual, Animation, ScreenEffect, RunTime, DiagnosticsTextBox, TextBox};
use dreamfield_renderer::resources::{ShaderManager, ModelManager, TextureManager, FontManager};
use dreamfield_macros::*;

use sim::*;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// The player position entering the village
const VILLAGE_ENTRANCE: (Vector3<f32>, Vector2<f32>) = (vec3(-125.1, 5.8, 123.8), vec2(0.063, -0.5));

/// The world chunks
const WORLD_CHUNKS: Dir<'_> = include_dir!("target/world_chunks");

/// Create the shader manager
pub fn create_shader_manager() -> ShaderManager {
    ShaderManager::new(vec![
        ("sky", preprocess_shader_vf!(include_bytes!("../resources/shaders/sky.glsl"))),
        ("ps1_no_tess", preprocess_shader_vf!(include_bytes!("../resources/shaders/ps1.glsl"))),
        ("ps1_tess", preprocess_shader_vtf!(include_bytes!("../resources/shaders/ps1.glsl"))),
        ("composite_yiq", preprocess_shader_vf!(include_bytes!("../resources/shaders/composite_yiq.glsl"))),
        ("composite_resolve", preprocess_shader_vf!(include_bytes!("../resources/shaders/composite_resolve.glsl"))),
        ("blit", preprocess_shader_vf!(include_bytes!("../resources/shaders/blit.glsl"))),
        ("text", preprocess_shader_vf!(include_bytes!("../resources/shaders/text.glsl"))),
    ])
}

/// Create the texture manager
pub fn create_texture_manager() -> TextureManager {
    TextureManager::new_with_textures(vec![
        ("sky", (include_bytes!("../resources/textures/sky.png"), TextureParams::repeat_nearest(), true, None))
    ])
}

/// Create the model manager
pub fn create_model_manager() -> ModelManager {
    ModelManager::new_with_models(vec![
        ("fire_orb", include_bytes!("../resources/models/fire_orb.glb")),
        ("tree", include_bytes!("../resources/models/tree.glb")),
        ("elf", include_bytes!("../resources/models/elf.glb")),
        ("minecart", include_bytes!("../resources/models/minecart.glb")),
    ])
}

/// Create the font manager
fn create_font_manager() -> FontManager {
    const MEDIEVAL_FONT_TEX: &'static [u8] = include_bytes!("../resources/fonts/0xDB_medievalish_chonker_8x8_1bpp_bmp_font_packed.png");
    const MEDIEVAL_FONT_MAP: &'static [u8] = include_bytes!("../resources/fonts/0xDB_medievalish_chonker_8x8_1bpp_bmp_font_packed.csv");
    FontManager::new(vec![
        ("medieval", MEDIEVAL_FONT_TEX, MEDIEVAL_FONT_MAP)
    ])
}

/// Create world entities
fn create_entities(world: &mut World) {
    // Diagnostics
    let stats_bounds = vec4(10.0, 10.0, 310.0, 230.0);
    world.spawn()
        .insert(DiagnosticsTextBox)
        .insert(TextBox::new("text", "medieval", "Vx8", "", None, Some(stats_bounds)));

    // Create sky
    world.spawn()
        .insert(ScreenEffect::new(RunTime::PreScene, "sky", Some("sky")));

    // Create player
    let (initial_pos, initial_rot) = VILLAGE_ENTRANCE;
    world.spawn()
        .insert(EntityName::new("Player"))
        // Entrance to village
        .insert(Transform::new(initial_pos, Matrix3::identity()))
        .insert(PlayerMovement::new_pos_look(PlayerMovementMode::Normal, initial_rot))
        .insert(PlayerMovement::collider())
        .insert(create_player_camera())
        .insert(EntitySpawnRadius::new(10.0));

    // Create fire orb
    world.spawn()
        .insert(FireOrb::default())
        .insert(Transform::new(vec3(-9.0, 0.0, 9.0), Matrix3::identity()))
        .insert(Visual::new_with_anim("fire_orb", false, Animation::Loop("Orb".to_string())));
}

/// Create the PlayerCamera with all our renderer params
fn create_player_camera() -> PlayerCamera {
    const RENDER_WIDTH: i32 = 320;
    const RENDER_HEIGHT: i32 = 240;

    const RENDER_ASPECT: f32 = 4.0 / 3.0;

    const FOV: f32 = 60.0;
    const NEAR_CLIP: f32 = 0.1;
    const FAR_CLIP: f32 = 35.0;

    const FOG_START: f32 = FAR_CLIP - 10.0;
    const FOG_END: f32 = FAR_CLIP - 5.0;

    const FOG_COLOR: Vector3<f32> = vec3(0.0, 0.0, 0.0);

    let proj = perspective(Deg(FOV), RENDER_ASPECT, NEAR_CLIP, FAR_CLIP);
    let view = Matrix4::identity();

    PlayerCamera {
        proj,
        view,
        render_res: vec2(RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
        render_aspect: RENDER_ASPECT,
        render_fov_rad: FOV * std::f32::consts::PI / 180.0,
        fog_color: FOG_COLOR,
        fog_range: vec2(FOG_START, FOG_END)
    }
}

/// Entry point
fn main() {
    // Initialise logging
    env_logger::init();
    log::info!("Welcome to Dreamfield!");

    // Create game host
    let mut host = GameHost::new(None, FIXED_UPDATE_TIME);

    // Create bevy world
    let mut world = World::default();

    // Initialise system and renderer
    dreamfield_system::init(&mut world);
    dreamfield_renderer::init(&mut world,
        create_model_manager(),
        create_shader_manager(),
        create_texture_manager(),
        create_font_manager(),
        WorldChunkManager::new(&WORLD_CHUNKS));

    // Create update schedule
    let mut update_schedule = Schedule::default();

    update_schedule.add_stage("sim", SystemStage::parallel()
        .with_system_set(dreamfield_system::systems())
        .with_system_set(sim::systems())
    );

    // Create render schedule
    let mut render_schedule = Schedule::default();

    render_schedule.add_stage("render", SystemStage::single_threaded()
        .with_system_set(dreamfield_renderer::systems())
    );

    // Initialise entities
    create_entities(&mut world);

    // Run game
    host.run(world, update_schedule, render_schedule);
}
