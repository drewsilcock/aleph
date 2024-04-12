use std::f32::consts::PI;

use bevy::math::bounding::{BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PrimaryWindow;
use rand::random;

use crate::AppState;
use crate::common::{cursor_world_coords, ParticleRadius, Velocity};

const BALL_COLOUR: Color = Color::rgb(1.0, 0.5, 0.5);
const BALL_RADIUS_MIN: f32 = 10.;
const BALL_RADIUS_MAX: f32 = 30.;
const NUM_BALLS: u32 = 100;
const BALL_SPACING: f32 = 10.;
const BALL_INITIAL_SPEED: f32 = 50.;

#[derive(Component)]
pub struct Ball;

fn initial_ball_velocity() -> Vec2 {
    (Vec2::new(random(), random()) * 2. - 1.) * BALL_INITIAL_SPEED
}

fn ball_radius() -> f32 {
    random::<f32>() * (BALL_RADIUS_MAX - BALL_RADIUS_MIN) + BALL_RADIUS_MIN
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                top: Val::Px(0.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "←",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans/FiraSans-Bold.ttf"),
                            font_size: 60.,
                            color: Color::rgb(0., 0., 0.),
                        },
                    ));
                });
        });

    let particles_per_row = f64::from(NUM_BALLS).sqrt();
    let particles_per_col = f64::from(NUM_BALLS - 1) / particles_per_row + 1.;
    let spacing = f64::from((BALL_RADIUS_MAX * 2.) + BALL_SPACING);

    for i in 0..NUM_BALLS {
        let x = (f64::from(i % particles_per_row as u32) - particles_per_row / 2. + 0.5) * spacing;
        let y = (f64::from(i) / particles_per_row - particles_per_col / 2.) * spacing;
        let radius = random::<f32>() * (BALL_RADIUS_MAX - BALL_RADIUS_MIN) + BALL_RADIUS_MIN;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(BALL_COLOUR),
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 1.))
                    .with_scale(Vec2::splat(radius * 2.).extend(1.)),
                ..default()
            },
            Ball,
            Velocity(initial_ball_velocity()),
            ParticleRadius(radius),
        ));
    }
}

pub fn destroy(
    mut commands: Commands,
    balls: Query<Entity, With<Ball>>,
    nodes: Query<Entity, With<Node>>,
) {
    for entity in balls.iter().chain(nodes.iter()) {
        commands.entity(entity).despawn();
    }
}

fn try_place_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    balls_query: Query<&Transform, With<Ball>>,
    position: Vec2,
) {
    let radius = ball_radius();
    let transform = Transform::from_translation(position.extend(1.))
        .with_scale(Vec2::splat(radius * 2.).extend(1.));

    let new_bounds = BoundingCircle::new(position, radius);
    for ball in balls_query.iter() {
        let bounds = BoundingCircle::new(ball.translation.truncate(), radius);
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
        Velocity(initial_ball_velocity()),
        ParticleRadius(radius),
    ));
}

// TODO: This is a narrow phase collision detection routine - we really need a broad phase one,
//  ideally with some spatial partitioning.
pub fn check_for_ball_collisions(
    mut query: Query<(&mut Transform, &mut Velocity, Option<&ParticleRadius>), With<Ball>>,
) {
    let mut iter = query.iter_combinations_mut();

    while let Some(
        [(mut left_transform, mut left_velocity, left_maybe_radius), (mut right_transform, mut right_velocity, right_maybe_radius)],
    ) = iter.fetch_next()
    {
        let left_radius = left_maybe_radius.map_or(0., |r| r.0);
        let right_radius = right_maybe_radius.map_or(0., |r| r.0);

        let left_bounds = BoundingCircle::new(left_transform.translation.truncate(), left_radius);
        let right_bounds =
            BoundingCircle::new(right_transform.translation.truncate(), right_radius);

        if left_bounds.intersects(&right_bounds) {
            // First, move the balls so they are no longer intersecting.
            let normal = (right_bounds.center - left_bounds.center).normalize();
            let distance = (left_bounds.center - right_bounds.center).length();
            let penetration = left_radius + right_radius - distance;
            let correction = penetration / 2. * normal;

            left_transform.translation -= correction.extend(0.);
            right_transform.translation += correction.extend(0.);

            // Currently assume all balls have same mass and radius, in the future this could change.
            // Assume all balls have the same density, so mass = volume = radius^2 * PI.
            let m1 = PI * left_radius.powi(2);
            let m2 = PI * right_radius.powi(2);
            let total_mass = m1 + m2;

            let relative_velocity = right_velocity.0 - left_velocity.0;
            let impulse_left = 2. * m1 / total_mass * relative_velocity.dot(normal) * normal;
            let impulse_right = 2. * m2 / total_mass * relative_velocity.dot(normal) * normal;

            left_velocity.0 += impulse_left;
            right_velocity.0 -= impulse_right;
        }
    }
}

pub fn handle_mouse_events(
    buttons: Res<ButtonInput<MouseButton>>,
    balls: Query<&Transform, With<Ball>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(position) = cursor_world_coords(window_query, camera_query) {
            try_place_ball(commands, meshes, materials, balls, position);
        }
    }
}

pub fn handle_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MainMenu);
        }
    }
}
