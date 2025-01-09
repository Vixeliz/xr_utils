use bevy::{prelude::*, render::primitives::Aabb};
use bevy_mod_xr::spaces::XrVelocity;
use bevy_rapier3d::prelude::*;

use crate::{
    prelude::{XrAction, XrInput, XrTrackedSpace},
    XrUtilsConfig,
};

#[derive(Component)]
/// Used to mark any entity you wan't holdable requires a velocity component, transform, and aabb
pub struct Grabbable;

#[derive(Component)]
/// Component to mark an entity as holding
pub struct Holding;

pub(crate) fn grab(
    mut commands: Commands,
    inputs: Option<Res<XrInput>>,
    hand_query: Query<
        (&Transform, &XrVelocity, Entity),
        (With<XrTrackedSpace>, Without<Holding>, Without<Grabbable>),
    >,
    mut holding_query: Query<
        (&mut Velocity, &mut Transform, &GlobalTransform, Entity),
        (With<Holding>, Without<XrTrackedSpace>),
    >,
    mut grabbable_query: Query<
        (&mut Transform, &Aabb, Entity),
        (
            With<Grabbable>,
            Without<XrTrackedSpace>,
            Without<XrVelocity>,
            Without<Holding>,
        ),
    >,
    rapier_context: Query<&RapierContext>,
    config: Res<XrUtilsConfig>,
) {
    if let Some((hand_transform, velocity, hand_entity)) = hand_query.iter().next() {
        let (action_name, action_type) = config.grab_action_names.first().unwrap();
        if let Some(input) = inputs {
            let input = input
                .state
                .get(&XrAction::from_string(action_name, action_type))
                .unwrap()
                .as_float()
                .unwrap();
            if let Ok((mut linear_vel, mut transform, global_transform, entity)) =
                holding_query.get_single_mut()
            {
                if input.cur_val <= 0.0 {
                    *transform = global_transform.compute_transform();
                    commands.entity(hand_entity).remove_children(&[entity]);
                    commands.entity(entity).remove::<Holding>();
                    commands.entity(entity).remove::<RigidBodyDisabled>();
                    linear_vel.linvel = velocity.linear;
                    return;
                }
            }

            for entity in rapier_context
                .get_single()
                .unwrap()
                .intersection_with_shape(
                    hand_transform.translation,
                    hand_transform.rotation,
                    &Collider::cuboid(0.1, 0.1, 0.05),
                    QueryFilter::only_dynamic(),
                )
                .iter()
            {
                if input.pressed {
                    if let Ok((mut transform, aabb, entity)) = grabbable_query.get_mut(*entity) {
                        let translation = -Vec3::new(aabb.half_extents.x, 0.0, aabb.half_extents.z)
                            - Vec3::new(0.025, 0.0, 0.0);
                        // - Vec3::new(0.05, 0.0, 0.0);
                        transform.translation = translation;
                        transform.rotation = Quat::IDENTITY;
                        commands.entity(entity).insert(Holding);
                        commands.entity(entity).insert(RigidBodyDisabled);
                        commands.entity(hand_entity).add_child(entity);
                    }
                }
            }
        }
    }
}
