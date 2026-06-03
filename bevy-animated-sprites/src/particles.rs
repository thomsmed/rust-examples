// Heavy inspired by Bevy's animated mesh events example
// https://github.com/bevyengine/bevy/blob/latest/examples/animation/animated_mesh_events.rs

use bevy::prelude::*;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, simulate_particles);
        app.insert_resource(SeededRng(ChaCha8Rng::seed_from_u64(1337)));
    }
}

#[derive(Resource)]
pub struct SeededRng(ChaCha8Rng);

#[derive(Component)]
pub struct Particle {
    lifetime_timer: Timer,
    size: f32,
    velocity: Vec3,
}

impl Particle {
    pub fn spawn_at(
        mut commands: Commands,
        mut rng: ResMut<SeededRng>,
        mesh: &Handle<Mesh>,
        material: &Handle<ColorMaterial>,
        location: Vec3,
    ) {
        for _ in 0..14 {
            let horizontal = rng.0.random_range(-8.0..4.0);
            let vertical = rng.0.random_range(0.0..2.0);
            let depth = rng.0.random_range(-1.0..1.0);
            let size = rng.0.random_range(0.2..1.0);

            commands.spawn((
                Particle {
                    lifetime_timer: Timer::from_seconds(
                        rng.0.random_range(0.2..0.6),
                        TimerMode::Once,
                    ),
                    size,
                    velocity: Vec3::new(horizontal, vertical, depth) * 10.0,
                },
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform {
                    translation: location,
                    scale: Vec3::splat(size),
                    ..Default::default()
                },
            ));
        }
    }
}

fn simulate_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in &mut query {
        if particle.lifetime_timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            return;
        }

        transform.translation += particle.velocity * time.delta_secs();
        transform.scale = Vec3::splat(particle.size.lerp(0.0, particle.lifetime_timer.fraction()));
        particle
            .velocity
            .smooth_nudge(&Vec3::ZERO, 4.0, time.delta_secs());
    }
}
