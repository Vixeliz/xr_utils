use bevy::prelude::*;
use bevy_mod_xr::spaces::XrVelocity;
use bevy_rapier3d::prelude::*;

use crate::{prelude::*, XrUtilsConfig};

#[derive(Component)]
/// Keep track of what entity we are currently gravity grabbing
pub struct GravityGrabbing;

#[derive(Component)]
/// Keep track of what entity we are currently targetting
pub struct Targetting;

// Calculate velocity to launch at hand FIX: When the object is really close to you it can cause issues
fn compute_velocity(hand_transform: Transform, obj_transform: Transform) -> Vec3 {
    let jump_angle = 60.0_f32.to_radians();
    let diff = hand_transform.translation - obj_transform.translation;
    let gravity_y = -9.81;
    let diffxz = Vec3::new(diff.x, 0.0, diff.z);
    let diffxz_length = diffxz.length();
    let diffy_length = diff.y;

    // Calculate jump speed
    let jumpspeed_unrooted = (-gravity_y * diffxz_length.powi(2))
        / (2.0 * jump_angle.cos().powi(2) * (diffxz_length * jump_angle.tan() - diffy_length));

    // prevent negative square roots
    let signum = jumpspeed_unrooted.signum();
    let jumpspeed = jumpspeed_unrooted.abs().sqrt() * signum;

    diffxz.normalize() * jump_angle.cos() * jumpspeed + (Vec3::Y * jump_angle.sin() * jumpspeed)
}

// Detecting if we should launch the entity and when TODO: Fix being able to infinitely float objects
pub(crate) fn gravity_grabbing(
    mut gravity_query: Query<
        (&mut Velocity, &mut Transform, Entity),
        (
            With<GravityGrabbing>,
            Without<Holding>,
            Without<XrTrackedSpace>,
        ),
    >,
    hand_query: Query<(&GlobalTransform, &XrVelocity), (With<XrTrackedSpace>, Without<Holding>)>,
    mut commands: Commands,
    config: Res<XrUtilsConfig>,
    inputs: Option<Res<XrInput>>,
) {
    if let Some((hand_transform, velocity)) = hand_query.iter().last() {
        let hand_transform = hand_transform.compute_transform();
        if let Ok((mut obj_velocity, obj_transform, entity)) = gravity_query.get_single_mut() {
            let (squeeze_action, pose_action) = config.gravity_grab_action_names.first().unwrap();
            if let Some(input) = inputs {
                let input = input.state.get(squeeze_action).unwrap().as_float().unwrap();
                if input.cur_val > 0.0 {
                    // Pick object with hand vel
                    obj_velocity.linvel = velocity.linear;
                    let threshold = 0.5;

                    // If we move to fast gravity grab
                    let magnitude = obj_velocity.linvel.length();
                    if magnitude > threshold {
                        let vel = compute_velocity(hand_transform, *obj_transform);
                        obj_velocity.linvel = vel;

                        commands.entity(entity).remove::<GravityGrabbing>();
                    }
                    return;
                }

                commands.entity(entity).remove::<GravityGrabbing>();
            }
        }
    }
}
// How we actuallly target entities
// TODO: make some stuff like max distance a resource for the plugin config
pub(crate) fn gesture(
    mut commands: Commands,
    hand_query: Query<(&GlobalTransform, &XrVelocity), (With<XrTrackedSpace>, Without<Holding>)>,
    mut gravity_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            // &Grabbable,
        ),
        (Without<Holding>, Without<XrTrackedSpace>, With<Grabbable>),
    >,
    holding_query: Query<&Holding>,
    gravity_grabbing: Query<&GravityGrabbing>,
    rapier_context: Query<&RapierContext>,
    config: Res<XrUtilsConfig>,
    inputs: Option<Res<XrInput>>,
) {
    if holding_query.get_single().is_ok() {
        return;
    }
    if gravity_grabbing.get_single().is_ok() {
        return;
    }
    if let Some((hand_transform, velocity)) = hand_query.iter().last() {
        let hand_transform = hand_transform.compute_transform();
        if let Some(hit) = rapier_context.get_single().unwrap().cast_shape(
            hand_transform.translation,
            Quat::IDENTITY,
            hand_transform.rotation.normalize() * -Vec3::Y,
            &Collider::ball(0.1),
            ShapeCastOptions {
                max_time_of_impact: 5.0,
                target_distance: 0.0,
                stop_at_penetration: false,
                compute_impact_geometry_on_penetration: false,
            },
            QueryFilter::only_dynamic(),
        ) {
            if let Ok((mut obj_velocity, transform)) = gravity_query.get_mut(hit.0) {
                // if **grabbable {
                let distance = hand_transform
                    .translation
                    .distance_squared(transform.translation);
                if distance <= 5.0 {
                    // So we can get whatever we are currently targetting
                    commands.entity(hit.0).insert(Targetting);
                    let (xr_action, pose_action) = config.grab_action_names.first().unwrap();
                    if let Some(input) = inputs {
                        let input = input.state.get(xr_action).unwrap().as_float().unwrap();

                        if input.pressed {
                            obj_velocity.linvel.y = velocity.linear.y;
                            commands.entity(hit.0).insert(GravityGrabbing);
                        }
                    }
                }
                // }
            }
        }
    }
}

pub(crate) fn outlines(mut commands: Commands, mut target_query: Query<Entity, With<Targetting>>) {
    for entity in target_query.iter_mut() {
        commands.entity(entity).remove::<Targetting>();
    }
}
