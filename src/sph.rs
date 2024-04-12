use std::f32::consts::PI;

use bevy::math::Vec2;
use bevy::prelude::{Query, Transform};

use crate::common::ParticleRadius;

fn smoothing_kernel(distance: f32, radius: f32) -> f32 {
    let volume = PI * radius.powi(8) / 4.;
    let val = 0_f32.max(radius * radius - distance * distance);
    val * val * val / volume
}

fn calculate_density(
    query: Query<(&Transform, Option<&ParticleRadius>)>,
    sample_point: Vec2,
) -> f32 {
    let mut density = 0_f32;
    let mass = 1_f32;

    // TODO: Only iterate over balls that are inside the smoothing radius.
    for (transform, maybe_radius) in query.iter() {
        let radius = maybe_radius.map_or(0., |r| r.0);
        let distance = transform.translation.truncate().distance(sample_point);
        let influence = smoothing_kernel(distance, radius / 2.);
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
