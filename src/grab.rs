use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
/// Used to mark any entity you wan't holdable requires a velocity component, transform, and aabb
pub struct Grabbable;

#[derive(Component)]
/// Component to mark an entity as holding
pub struct Holding;

pub(crate) fn grab(
    mut commands: Commands,
    just_pressed: Res<JustPressedGrip>,
    hand_query: Query<
        (&Transform, &XrVelocity, Entity),
        (
            With<XrTrackedRightGrip>,
            Without<Holding>,
            Without<Grabbable>,
        ),
    >,
    mut holding_query: Query<
        (&mut Velocity, &mut Transform, &Aabb, Entity),
        (With<Holding>, Without<XrTrackedRightGrip>),
    >,
    mut grabbable_query: Query<
        (&mut Transform, &Aabb, Entity),
        (
            With<Grabbable>,
            Without<XrTrackedRightGrip>,
            Without<XrVelocity>,
            Without<Holding>,
        ),
    >,
    rapier_context: Query<&RapierContext>,
) {
    if let Ok((hand_transform, velocity, hand_entity)) = hand_query.get_single() {
        if let Ok((mut linear_vel, mut transform, aabb, entity)) = holding_query.get_single_mut() {
            if just_pressed.cur_val <= 0.0 {
                let translation = hand_transform.translation
                    - Vec3::new(aabb.half_extents.x, 0.0, aabb.half_extents.z)
                    - Vec3::new(0.025, 0.0, 0.0);
                transform.translation = translation;
                transform.rotation = hand_transform.rotation;
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
            if just_pressed.pressed {
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
