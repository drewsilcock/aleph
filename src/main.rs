#![allow(clippy::needless_pass_by_value)]

use std::f32::consts::PI;

use bevy::math::bounding::{BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PrimaryWindow;
use rand::random;

const GRAVITY: f32 = -600.;
const BACKGROUND_COLOUR: Color = Color::rgb(0.9, 0.9, 0.9);
const BALL_COLOUR: Color = Color::rgb(1.0, 0.5, 0.5);
const BOUNCE_DAMPING: f32 = 0.6;
const BALL_DIAMETER: f32 = 30.;
const NUM_BALLS: u32 = 100;
const BALL_SPACING: f32 = 10.;
const BALL_INITIAL_SPEED: f32 = 50.;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn smoothing_kernel(distance: f32, radius: f32) -> f32 {
    let volume = PI * radius.powi(8) / 4.;
    let val = 0_f32.max(radius * radius - distance * distance);
    val * val * val / volume
}

fn calculate_density(positions_query: Query<&Transform>, sample_point: Vec2) -> f32 {
    let mut density = 0_f32;
    let mass = 1_f32;

    // TODO: Only iterate over balls that are inside the smoothing radius.
    for transform in positions_query.iter() {
        let distance = transform.translation.truncate().distance(sample_point);
        let influence = smoothing_kernel(distance, BALL_DIAMETER);
        density += mass * influence;
    }

    density
}

/*
fn calculate_property<T: Component + Into<f32> + Deref + DerefMut + Clone>(
    mut query: Query<(&Transform, &T)>,
    sample_point: Vec2,
) -> f32 {
    let transform_query = {
        let mut lens = query.transmute_lens::<&Transform>();
        lens.query()
    };
    let mut property = 0_f32;
    let mass = 1_f32;

    for (transform, particleProperty) in query.iter() {
        let distance = transform.translation.truncate().distance(sample_point);
        let influence = smoothing_kernel(distance, BALL_DIAMETER);
        let density = calculate_density(transform_query, sample_point);
        let particle_property_value: f32 = particleProperty.clone().into();
        property += particle_property_value * mass / density * influence;
    }

    property
}
 */

fn initial_velocity() -> Vec2 {
    (Vec2::new(random(), random()) * 2. - 1.) * BALL_INITIAL_SPEED
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let particles_per_row = f64::from(NUM_BALLS).sqrt();
    let particles_per_col = f64::from(NUM_BALLS - 1) / particles_per_row + 1.;
    let spacing = f64::from(BALL_DIAMETER + BALL_SPACING);

    for i in 0..NUM_BALLS {
        let x = (f64::from(i % particles_per_row as u32) - particles_per_row / 2. + 0.5) * spacing;
        let y = (f64::from(i) / particles_per_row - particles_per_col / 2.) * spacing;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(BALL_COLOUR),
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 1.))
                    .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
                ..default()
            },
            Ball,
            Velocity(initial_velocity()),
        ));
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn apply_gravity(
    mut query: Query<(&mut Velocity, &Transform), With<Ball>>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    let window = windows.single();

    for (mut velocity, position) in &mut query {
        if position.translation.y - BALL_DIAMETER / 2. <= -window.height() / 2. {
            // Once ball is on the floor, floor pushes back up.
        } else {
            velocity.y += GRAVITY * time.delta_seconds();
        }
    }
}

fn cursor_world_coords(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    let cursor_logical_px = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_logical_px)?;
    Some(ray.origin.truncate())
}

fn try_place_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    balls_query: Query<&Transform, With<Ball>>,
    position: Vec2,
) {
    let transform = Transform::from_translation(position.extend(1.))
        .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.));

    let new_bounds = BoundingCircle::new(position, BALL_DIAMETER / 2.);
    for ball in balls_query.iter() {
        let bounds = BoundingCircle::new(ball.translation.truncate(), BALL_DIAMETER / 2.);
        if new_bounds.intersects(&bounds) {
            return;
        }
    }

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOUR),
            transform,
            ..default()
        },
        Ball,
        Velocity(initial_velocity()),
    ));
}

fn handle_mouse_events(
    buttons: Res<ButtonInput<MouseButton>>,
    balls: Query<&Transform, With<Ball>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = cursor_world_coords(window_query, camera_query) {
            try_place_ball(commands, meshes, materials, balls, position);
        }
    }
}

fn check_for_boundary_collisions(
    mut ball_query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let half_bounds =
        Vec2::new(window.width(), window.height()) / 2. - Vec2::splat(BALL_DIAMETER / 2.);

    for (mut velocity, mut transform) in &mut ball_query.iter_mut() {
        if transform.translation.x.abs() > half_bounds.x {
            transform.translation.x = half_bounds.x * transform.translation.x.signum();
            velocity.x *= -BOUNCE_DAMPING;
        }

        if transform.translation.y.abs() > half_bounds.y {
            transform.translation.y = half_bounds.y * transform.translation.y.signum();
            velocity.y *= -BOUNCE_DAMPING;
        }
    }
}

fn check_for_ball_collisions(mut query: Query<(&mut Transform, &mut Velocity), With<Ball>>) {
    let mut iter = query.iter_combinations_mut();

    while let Some(
        [(mut left_transform, mut left_velocity), (mut right_transform, mut right_velocity)],
    ) = iter.fetch_next()
    {
        let left_bounds =
            BoundingCircle::new(left_transform.translation.truncate(), BALL_DIAMETER / 2.);
        let right_bounds =
            BoundingCircle::new(right_transform.translation.truncate(), BALL_DIAMETER / 2.);

        if left_bounds.intersects(&right_bounds) {
            // First, move the balls so they are no longer intersecting.
            let normal = (right_bounds.center - left_bounds.center).normalize();
            let distance = (left_bounds.center - right_bounds.center).length();
            let penetration = BALL_DIAMETER - distance;
            let correction = penetration / 2. * normal;

            left_transform.translation -= correction.extend(0.);
            right_transform.translation += correction.extend(0.);

            // Currently assume all balls have same mass and radius, in the future this could change.
            let m1 = 1.;
            let m2 = 1.;
            let total_mass = m1 + m2;

            let relative_velocity = right_velocity.0 - left_velocity.0;
            let impulse_left = 2. * m1 / total_mass * relative_velocity.dot(normal) * normal;
            let impulse_right = 2. * m2 / total_mass * relative_velocity.dot(normal) * normal;

            left_velocity.0 += impulse_left;
            right_velocity.0 -= impulse_right;
        }
    }
}

pub struct AlephPlugin;

impl Plugin for AlephPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOUR))
            .add_systems(Startup, setup)
            .add_systems(Update, handle_mouse_events)
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_gravity,
                    check_for_boundary_collisions,
                    check_for_ball_collisions,
                )
                    .chain(),
            );
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins, AlephPlugin)).run();
}
