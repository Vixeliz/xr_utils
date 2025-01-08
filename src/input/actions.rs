use bevy::{prelude::*, utils::HashMap};
use bevy_mod_openxr::{
    action_binding::OxrSuggestActionBinding,
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::OxrSyncActionSet,
    helper_traits::{ToQuat, ToVec3},
    resources::{OxrFrameState, OxrInstance, Pipelined},
    session::OxrSession,
    spaces::OxrSpaceLocationFlags,
};
use bevy_mod_xr::{
    session::{XrTracker, XrTrackingRoot},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace, XrVelocity},
};
use openxr::{Posef, Vector2f};
// use openxr::{Action, Posef, Vector2f};
use serde::{Deserialize, Serialize};

// #[derive(Resource)]
// pub struct XrActions {
//     HashMap
// }

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Component)]
pub struct XrAction {
    name: String,
    pretty_name: String,
    action_type: XrActionType,
}

impl std::hash::Hash for XrAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct XrBinding {
    interaction_profile: String,
    binding: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    bindings: Vec<(XrAction, XrBinding)>,
    set_name: String,
    set_pretty_name: String,
}

#[derive(Resource)]
pub struct XrActions {
    set: openxr::ActionSet,
    actions: HashMap<XrAction, XrRawActionState>,
    config: Config,
}

#[derive(Resource)]
pub struct XrInput {
    state: HashMap<XrAction, XrActionState>,
}

#[derive(Component)]
pub struct HeadXRSpace(XrReferenceSpace);

#[derive(Component)]
pub struct XrTrackedStage;

#[derive(Component)]
pub struct XrTrackedLocalFloor;

#[derive(Component)]
pub struct XrTrackedView;

#[derive(Component)]
pub struct XrSpace;

#[derive(Component)]
pub struct XrTrackedSpace(XrAction);

pub fn spawn_tracking_rig(actions: Res<XrActions>, mut cmds: Commands, session: Res<OxrSession>) {
    //head
    let head_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, Transform::IDENTITY)
        .unwrap();
    cmds.spawn((
        Transform::default(),
        Visibility::default(),
        XrTracker,
        XrVelocity::new(),
        HeadXRSpace(head_space),
    ));

    for action in actions.actions.iter().by_ref() {
        match action.1 {
            XrRawActionState::Pose(x) => {
                let space = session
                    .create_action_space(x, openxr::Path::NULL, Isometry3d::IDENTITY)
                    .unwrap();
                cmds.spawn((space, XrSpace, XrVelocity::new(), action.0.clone()));
            }
            _ => {}
        }
    }
}

pub fn update_spaces(
    mut space_query: Query<(&mut Transform, &XrAction), (With<XrSpace>, Without<XrTrackedSpace>)>,
    mut tracked_space_query: Query<
        (&mut Transform, Option<&XrAction>, Entity),
        (With<XrTrackedSpace>, Without<XrSpace>),
    >,
    mut cmds: Commands,
) {
    for (space_transform, space_action) in space_query.iter_mut() {
        // if let Ok(space) = space_transform {
        for (mut transform, action, entity) in &mut tracked_space_query {
            *transform = *space_transform;
            if action.is_none() {
                cmds.entity(entity).insert(space_action.clone());
            }
        }
        // }
    }
}

//stage
pub fn update_stage(
    root_query: Query<&Transform, (With<XrTrackingRoot>, Without<XrTrackedStage>)>,
    mut stage_query: Query<&mut Transform, (With<XrTrackedStage>, Without<XrTrackingRoot>)>,
) {
    if let Ok(root) = root_query.get_single() {
        for mut transform in &mut stage_query {
            *transform = *root;
        }
    }
}

pub fn update_head_transforms(
    session: Res<OxrSession>,
    default_ref_space: Res<XrPrimaryReferenceSpace>,
    pipelined: Option<Res<Pipelined>>,
    frame_state: Res<OxrFrameState>,
    mut query: Query<(&mut Transform, &HeadXRSpace, Option<&XrReferenceSpace>)>,
) {
    for (mut transform, space, ref_space) in &mut query {
        let ref_space = ref_space.unwrap_or(&default_ref_space);
        let time = if pipelined.is_some() {
            openxr::Time::from_nanos(
                frame_state.predicted_display_time.as_nanos()
                    + frame_state.predicted_display_period.as_nanos(),
            )
        } else {
            frame_state.predicted_display_time
        };
        let space_location = session.locate_space(&space.0, ref_space, time);

        if let Ok(space_location) = space_location {
            let flags = OxrSpaceLocationFlags(space_location.location_flags);
            if flags.pos_valid() {
                transform.translation = space_location.pose.position.to_vec3();
            }
            if flags.rot_valid() {
                transform.rotation = space_location.pose.orientation.to_quat();
            }
        }
    }
}

pub fn update_view(
    mut head_query: Query<&mut Transform, (With<HeadXRSpace>, Without<XrTrackedView>)>,
    mut view_query: Query<&mut Transform, (With<XrTrackedView>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_query.get_single_mut();
    if let Ok(root) = head_transform {
        for mut transform in &mut view_query {
            *transform = *root;
        }
    }
}

pub fn update_local_floor_transforms(
    mut head_space: Query<&mut Transform, (With<HeadXRSpace>, Without<XrTrackedLocalFloor>)>,
    mut local_floor: Query<&mut Transform, (With<XrTrackedLocalFloor>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_space.get_single_mut();
    if let Ok(head) = head_transform {
        let mut calc_floor = *head;
        calc_floor.translation.y = 0.0;
        //TODO: use yaw
        let (y, _, _) = calc_floor.rotation.to_euler(EulerRot::YXZ);
        let new_rot = Quat::from_rotation_y(y);
        calc_floor.rotation = new_rot;
        for mut transform in &mut local_floor {
            *transform = calc_floor;
        }
    }
}

pub fn create_input(actions: Res<XrActions>, mut cmds: Commands, session: Res<OxrSession>) {}

pub fn sync_actions(actions: Res<XrActions>, mut sync: EventWriter<OxrSyncActionSet>) {
    sync.send(OxrSyncActionSet(actions.set.clone()));
}

pub fn attach_set(actions: Res<XrActions>, mut attach: EventWriter<OxrAttachActionSet>) {
    attach.send(OxrAttachActionSet(actions.set.clone()));
}

pub fn suggest_action_bindings(
    actions: Res<XrActions>,
    mut bindings: EventWriter<OxrSuggestActionBinding>,
) {
    for binding in actions.config.bindings.clone() {
        match actions.actions.get(&binding.0).unwrap() {
            XrRawActionState::Float(x) => {
                bindings.send(OxrSuggestActionBinding {
                    action: x.as_raw(),
                    interaction_profile: binding.1.interaction_profile.into(),
                    bindings: binding
                        .1
                        .binding
                        .iter()
                        .cloned()
                        .map(|a| Into::<std::borrow::Cow<'static, str>>::into(a))
                        .collect(),
                });
            }
            XrRawActionState::Bool(x) => {
                bindings.send(OxrSuggestActionBinding {
                    action: x.as_raw(),
                    interaction_profile: binding.1.interaction_profile.into(),
                    bindings: binding
                        .1
                        .binding
                        .iter()
                        .cloned()
                        .map(|a| Into::<std::borrow::Cow<'static, str>>::into(a))
                        .collect(),
                });
            }
            XrRawActionState::Pose(x) => {
                bindings.send(OxrSuggestActionBinding {
                    action: x.as_raw(),
                    interaction_profile: binding.1.interaction_profile.into(),
                    bindings: binding
                        .1
                        .binding
                        .iter()
                        .cloned()
                        .map(|a| Into::<std::borrow::Cow<'static, str>>::into(a))
                        .collect(),
                });
            }
            XrRawActionState::Vec2(x) => {
                bindings.send(OxrSuggestActionBinding {
                    action: x.as_raw(),
                    interaction_profile: binding.1.interaction_profile.into(),
                    bindings: binding
                        .1
                        .binding
                        .iter()
                        .cloned()
                        .map(|a| Into::<std::borrow::Cow<'static, str>>::into(a))
                        .collect(),
                });
            }
        }
    }
}

impl XrActions {
    fn from_config(config: Config, instance: &OxrInstance) -> Self {
        let set = instance
            .create_action_set(config.set_name.as_str(), config.set_pretty_name.as_str(), 0)
            .unwrap();

        let mut actions = HashMap::new();
        for binding in config.bindings.clone() {
            match binding.0.action_type {
                XrActionType::Float => {
                    actions.insert(
                        binding.0.clone(),
                        XrRawActionState::Float(
                            set.create_action::<f32>(
                                binding.0.name.as_str(),
                                binding.0.pretty_name.as_str(),
                                &[],
                            )
                            .unwrap(),
                        ),
                    );
                }
                XrActionType::Bool => {
                    actions.insert(
                        binding.0.clone(),
                        XrRawActionState::Bool(
                            set.create_action::<bool>(
                                binding.0.name.as_str(),
                                binding.0.pretty_name.as_str(),
                                &[],
                            )
                            .unwrap(),
                        ),
                    );
                }
                XrActionType::Pose => {
                    actions.insert(
                        binding.0.clone(),
                        XrRawActionState::Pose(
                            set.create_action::<Posef>(
                                binding.0.name.as_str(),
                                binding.0.pretty_name.as_str(),
                                &[],
                            )
                            .unwrap(),
                        ),
                    );
                }
                XrActionType::Vec2 => {
                    actions.insert(
                        binding.0.clone(),
                        XrRawActionState::Vec2(
                            set.create_action::<Vector2f>(
                                binding.0.name.as_str(),
                                binding.0.pretty_name.as_str(),
                                &[],
                            )
                            .unwrap(),
                        ),
                    );
                }
            }
        }

        Self {
            set,
            actions,
            config: config.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone)]
pub enum XrActionType {
    Float,
    Vec2,
    Bool,
    Pose,
}

pub enum XrActionState {
    Float(f32),
    Vec2(Vec2),
    Bool(bool),
    Pose(Transform),
}

pub enum XrRawActionState {
    Float(openxr::Action<f32>),
    Vec2(openxr::Action<openxr::Vector2f>),
    Bool(openxr::Action<bool>),
    Pose(openxr::Action<openxr::Posef>),
}
