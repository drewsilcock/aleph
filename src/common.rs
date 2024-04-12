use bevy::math::Vec2;
use bevy::prelude::{
    Camera, Component, Deref, DerefMut, GlobalTransform, Query, Res, Time, Transform, Window, With,
};
use bevy::window::PrimaryWindow;

const GRAVITY: f32 = -800.;
const BOUNDARY_COLLISION_DAMPING: f32 = 0.6;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Deref, DerefMut)]
pub struct ParticleRadius(pub f32);

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn apply_gravity(
    mut query: Query<(&mut Velocity, &Transform, Option<&ParticleRadius>)>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    let window = windows.single();

    for (mut velocity, transform, maybe_radius) in &mut query {
        let radius = maybe_radius.map_or(0., |r| r.0);
        if transform.translation.y - radius <= -window.height() / 2. {
            // Once ball is on the floor, floor pushes back up.
        } else {
            velocity.y += GRAVITY * time.delta_seconds();
        }
    }
}

pub fn check_for_boundary_collisions(
    mut ball_query: Query<(&mut Velocity, &mut Transform, Option<&ParticleRadius>)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let half_bounds = Vec2::new(window.width(), window.height()) / 2.;

    for (mut velocity, mut transform, maybe_radius) in &mut ball_query.iter_mut() {
        let radius = maybe_radius.map_or(0., |r| r.0);

        if transform.translation.x.abs() > (half_bounds.x - radius) {
            transform.translation.x = (half_bounds.x - radius) * transform.translation.x.signum();
            velocity.x *= -BOUNDARY_COLLISION_DAMPING;
        }

        if transform.translation.y.abs() > (half_bounds.y - radius) {
            transform.translation.y = (half_bounds.y - radius) * transform.translation.y.signum();
            velocity.y *= -BOUNDARY_COLLISION_DAMPING;
        }
    }
}

pub fn cursor_world_coords(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();

    let cursor_logical_px = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_logical_px)?;
    Some(ray.origin.truncate())
}
