use crate::animated_sprite::{AnimatedSprite, AnimatedSpritePlugin, TextureAtlasIndex};
use bevy::animation::{AnimatedBy, AnimationEvent, AnimationTargetId, animated_field};
use bevy::color::palettes::basic::GREEN;
use bevy::prelude::*;
use std::f32::consts::PI;

mod animated_sprite;

#[derive(Resource)]
struct CharacterAnimations {
    run: AnimationNodeIndex,
}

#[derive(AnimationEvent, Clone)]
struct CharacterStep;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(
            ImagePlugin::default_nearest(), // Makes sprites/images crisp
        ))
        .add_plugins(AnimatedSpritePlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_ground)
        .add_systems(Startup, setup_character)
        .add_systems(FixedUpdate, toggle_animation)
        .add_observer(on_character_step)
        .run()
}

fn setup_camera(
    mut commands: Commands,
) {
    // # Camera
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Transform::default()
            .with_translation(Vec3::new(0.0, 32.0, 0.0))
            .with_scale(Vec3::splat(0.2)),
    ));
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(512.0, 2.0)))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(GREEN))),
        Transform::from_xyz(0.0, -12.0, 0.0),
    ));
}

fn setup_character(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // # Character entity
    let character_name = Name::new("Character");
    let character_entity = commands.spawn(character_name.clone()).id();

    let sprite_sheet_num_rows = 1;
    let sprite_sheet_num_columns = 7;
    let sprite_sheet_section_size = 24;

    let texture_handle = asset_server.load("gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(sprite_sheet_section_size),
        sprite_sheet_num_columns,
        sprite_sheet_num_rows,
        None,
        None,
    );
    let texture_atlas_layout_handle = texture_atlas_layouts.add(texture_atlas_layout);

    let initial_section_index = 0;
    commands.entity(character_entity).insert((
        character_name.clone(),
        Sprite::from_atlas_image(
            texture_handle.clone(),
            TextureAtlas {
                layout: texture_atlas_layout_handle.clone(),
                index: initial_section_index,
            },
        ),
        AnimatedSprite::from_index(initial_section_index),
    ));

    // # Character animations
    let character_animation_target_id = AnimationTargetId::from_name(&character_name);
    let mut character_animation_graph = AnimationGraph::new();

    // ## Character run animation
    const CHARACTER_RUN_ANIMATION_DURATION: f32 = 0.6; // 0.6 seconds
    let mut character_run_animation_clip = AnimationClip::default();

    // ### Animate character sprite
    const CHARACTER_RUN_SECONDS_PER_FRAME: f32 = CHARACTER_RUN_ANIMATION_DURATION / 6.0;
    let character_run_keyframe_curve = AnimatableKeyframeCurve::new([
        (0.0, TextureAtlasIndex::new(1)), // Foot touches ground
        (CHARACTER_RUN_SECONDS_PER_FRAME * 1.0, TextureAtlasIndex::new(2)),
        (CHARACTER_RUN_SECONDS_PER_FRAME * 2.0, TextureAtlasIndex::new(3)),
        (CHARACTER_RUN_SECONDS_PER_FRAME * 3.0, TextureAtlasIndex::new(4)), // Foot touches ground
        (CHARACTER_RUN_SECONDS_PER_FRAME * 4.0, TextureAtlasIndex::new(5)),
        (CHARACTER_RUN_SECONDS_PER_FRAME * 5.0, TextureAtlasIndex::new(6)),
        (CHARACTER_RUN_ANIMATION_DURATION, TextureAtlasIndex::new(6)),
    ])
    .expect("Should be valid keyframes");
    let character_run_sprite_animation_curve = AnimatableCurve::new(
        animated_field!(AnimatedSprite::current_index),
        character_run_keyframe_curve,
    );
    character_run_animation_clip.add_curve_to_target(
        character_animation_target_id,
        character_run_sprite_animation_curve,
    );

    // ### Animate character transform
    let character_run_bounce_curve = FunctionCurve::new(
        Interval::new(0.0, CHARACTER_RUN_ANIMATION_DURATION).unwrap(),
        |t| {
            // y = 1 + cos(PI + t * K),
            // where K is some constant making y(0) = 0.0 and y(CHARACTER_RUN_ANIMATION_DURATION) = 0.0,
            // meaning PI + t * K = PI for t = 0.0,
            // and PI + t * K = 5PI for t = CHARACTER_RUN_ANIMATION_DURATION.
            // That gives: K = 4PI / CHARACTER_RUN_ANIMATION_DURATION
            const K: f32 = (4.0 * PI) / CHARACTER_RUN_ANIMATION_DURATION;
            Vec3::new(0.0, 1.0 + (PI + t * K).cos(), 0.0)
        },
    );
    let character_run_bounce_animation_curve = AnimatableCurve::new(
        animated_field!(Transform::translation),
        character_run_bounce_curve,
    );
    character_run_animation_clip.add_curve_to_target(
        character_animation_target_id,
        character_run_bounce_animation_curve,
    );

    // ### Animation event(s)
    character_run_animation_clip.add_event(
        0.0, // Matches sprite frame where foot touches ground
        CharacterStep,
    );
    character_run_animation_clip.add_event(
        CHARACTER_RUN_SECONDS_PER_FRAME * 3.0, // Matches sprite frame where foot touches ground
        CharacterStep,
    );

    // ## Remember animation node indices
    let character_run_animation_node_index = character_animation_graph.add_clip(
        animations.add(character_run_animation_clip),
        0.0,
        character_animation_graph.root,
    );

    // ## Register animation clip
    commands.insert_resource(CharacterAnimations {
        run: character_run_animation_node_index,
    });

    // ## Animation target
    commands.entity(character_entity).insert((
        character_animation_target_id,
        AnimatedBy(character_entity), // The character entity animates itself
    ));

    // ## Animation player (often not the same as the target, but in our case it is)
    commands.entity(character_entity).insert((
        AnimationPlayer::default(),
        AnimationGraphHandle(graphs.add(character_animation_graph)),
    ));
}

fn on_character_step(event: On<CharacterStep>) {
    info!("on_character_step");
}

fn toggle_animation(
    animation_players: Query<&mut AnimationPlayer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    character_animations: Res<CharacterAnimations>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut animation_player in animation_players {
            if animation_player.is_playing_animation(character_animations.run) {
                animation_player.stop_all();
            } else {
                animation_player.play(character_animations.run).repeat();
            }
        }
    }
}
