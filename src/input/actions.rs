use bevy::{prelude::*, utils::HashMap};
use bevy_mod_openxr::{
    action_binding::OxrSuggestActionBinding,
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::OxrSyncActionSet,
    helper_traits::{ToQuat, ToVec2, ToVec3},
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

#[derive(Deserialize, Serialize, Clone, Component, Debug)]
pub struct XrAction {
    pub name: String,
    pub pretty_name: String,
    pub action_type: XrActionType,
}

// FIX: THIS IS JANK like all my code
impl XrAction {
    /// Does not make a proper action should only be used for getting items from xractions mainly missing pretty name
    pub fn from_string(string: &String, action_type: &XrActionType) -> Self {
        Self {
            name: string.clone(),
            pretty_name: string.clone(),
            action_type: action_type.clone(),
        }
    }
}

impl std::hash::Hash for XrAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for XrAction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for XrAction {}

#[derive(Deserialize, Serialize, Clone)]
pub struct XrBinding {
    interaction_profile: String,
    binding: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Resource)]
pub struct Config {
    bindings: Vec<(XrAction, XrBinding)>,
    set_name: String,
    set_pretty_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bindings: vec![
                (
                    XrAction {
                        name: "right_pose".to_string(),
                        pretty_name: "Right Hand Grip Pose".to_string(),
                        action_type: XrActionType::Pose,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/right/input/grip/pose".into()],
                    },
                ),
                (
                    XrAction {
                        name: "left_pose".to_string(),
                        pretty_name: "Left Hand Grip Pose".to_string(),
                        action_type: XrActionType::Pose,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/left/input/grip/pose".into()],
                    },
                ),
                (
                    XrAction {
                        name: "left_joystick".to_string(),
                        pretty_name: "Left Hand JoyStick".to_string(),
                        action_type: XrActionType::Vec2,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/left/input/thumbstick".into()],
                    },
                ),
                (
                    XrAction {
                        name: "right_joystick".to_string(),
                        pretty_name: "Right Hand JoyStick".to_string(),
                        action_type: XrActionType::Vec2,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/right/input/thumbstick".into()],
                    },
                ),
                (
                    XrAction {
                        name: "right_squeeze".to_string(),
                        pretty_name: "Right Hand Squeeze".to_string(),
                        action_type: XrActionType::Float,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/right/input/squeeze/value".into()],
                    },
                ),
                (
                    XrAction {
                        name: "left_squeeze".to_string(),
                        pretty_name: "Left Hand Squeeze".to_string(),
                        action_type: XrActionType::Float,
                    },
                    XrBinding {
                        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
                        binding: vec!["/user/hand/left/input/squeeze/value".into()],
                    },
                ),
            ],
            set_name: "mine".to_string(),
            set_pretty_name: "My set".to_string(),
        }
    }
}

#[derive(Resource)]
pub struct XrActions {
    set: openxr::ActionSet,
    actions: HashMap<XrAction, XrRawActionState>,
}

#[derive(Resource, Debug)]
pub struct XrInput {
    pub state: HashMap<XrAction, XrActionState>,
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
pub struct XrTrackedSpace;

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

    for action in actions.actions.iter() {
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

pub fn update_inputs(
    inputs: Option<ResMut<XrInput>>,
    actions: Option<Res<XrActions>>,
    session: Option<Res<OxrSession>>,
) {
    if let Some(mut inputs) = inputs {
        if let Some(session) = session {
            if let Some(actions) = actions {
                for action in actions.actions.iter() {
                    match action.1 {
                        XrRawActionState::Float(x) => {
                            if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                                if let Some(prev_value) = inputs.state.get_mut(&action.0.clone()) {
                                    let prev_value = prev_value.as_float_mut().unwrap();
                                    prev_value.pressed = prev_value.previous_val <= 0.0
                                        && action_new.current_state > 0.0;
                                    prev_value.cur_val = action_new.current_state;
                                }
                            }
                        }
                        XrRawActionState::Bool(x) => {
                            if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                                if let Some(prev_value) = inputs.state.get_mut(&action.0.clone()) {
                                    let prev_value = prev_value.as_bool_mut().unwrap();
                                    prev_value.pressed =
                                        !prev_value.previous_val && action_new.current_state;
                                    prev_value.cur_val = action_new.current_state;
                                }
                            }
                        }
                        XrRawActionState::Vec2(x) => {
                            if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                                if let Some(prev_value) = inputs.state.get_mut(&action.0.clone()) {
                                    let prev_value = prev_value.as_vec2_mut().unwrap();
                                    prev_value.cur_val = action_new.current_state.to_vec2();
                                    prev_value.pressed_x = prev_value.previous_val.x == 0.0
                                        && action_new.current_state.x != 0.0;
                                    prev_value.pressed_y = prev_value.previous_val.y == 0.0
                                        && action_new.current_state.y != 0.0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn end_frame_input(inputs: Option<ResMut<XrInput>>) {
    if let Some(mut inputs) = inputs {
        for input in inputs.state.iter_mut() {
            match input.1 {
                XrActionState::Float(x) => {
                    x.previous_val = x.cur_val;
                }
                XrActionState::Bool(x) => {
                    x.previous_val = x.cur_val;
                }
                XrActionState::Vec2(x) => {
                    x.previous_val = x.cur_val;
                }
            }
        }
    }
}

pub fn update_spaces(
    mut space_query: Query<
        (&mut Transform, &XrAction, &XrVelocity),
        (With<XrSpace>, Without<XrTrackedSpace>),
    >,
    mut tracked_space_query: Query<
        (&mut Transform, &XrAction, &mut XrVelocity),
        (With<XrTrackedSpace>, Without<XrSpace>),
    >,
) {
    for (space_transform, space_action, space_velocity) in space_query.iter_mut() {
        for (mut transform, action, mut velocity) in tracked_space_query.iter_mut() {
            if action == space_action {
                *transform = *space_transform;
                *velocity = *space_velocity;
            }
        }
    }
}

//stage
pub fn update_stage(
    root_query: Query<&Transform, (With<XrTrackingRoot>, Without<XrTrackedStage>)>,
    mut stage_query: Query<&mut Transform, (With<XrTrackedStage>, Without<XrTrackingRoot>)>,
) {
    if let Ok(root) = root_query.get_single() {
        for mut transform in stage_query.iter_mut() {
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

pub fn create_input(actions: Res<XrActions>, mut cmds: Commands, session: Res<OxrSession>) {
    let mut xr_input = XrInput {
        state: HashMap::new(),
    };
    for action in actions.actions.iter() {
        match action.1 {
            XrRawActionState::Float(x) => {
                if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                    xr_input.state.insert(
                        action.0.clone(),
                        XrActionState::Float(XrActionStateFloat {
                            previous_val: 0.0,
                            cur_val: action_new.current_state,
                            pressed: false,
                        }),
                    );
                    // .unwrap();
                }
            }
            XrRawActionState::Bool(x) => {
                if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                    xr_input.state.insert(
                        action.0.clone(),
                        XrActionState::Bool(XrActionStateBool {
                            previous_val: false,
                            cur_val: action_new.current_state,
                            pressed: false,
                        }),
                    );
                }
            }
            XrRawActionState::Vec2(x) => {
                if let Ok(action_new) = x.state(&session, openxr::Path::NULL) {
                    xr_input.state.insert(
                        action.0.clone(),
                        XrActionState::Vec2(XrActionStateVec2 {
                            previous_val: Vec2::ZERO,
                            cur_val: action_new.current_state.to_vec2(),
                            pressed_x: false,
                            pressed_y: false,
                        }),
                    );
                }
            }
            _ => {}
        }
    }
    cmds.insert_resource(xr_input);
}

pub fn sync_actions(actions: Res<XrActions>, mut sync: EventWriter<OxrSyncActionSet>) {
    sync.send(OxrSyncActionSet(actions.set.clone()));
}

pub fn attach_set(actions: Res<XrActions>, mut attach: EventWriter<OxrAttachActionSet>) {
    attach.send(OxrAttachActionSet(actions.set.clone()));
}

pub fn suggest_action_bindings(
    actions: Res<XrActions>,
    config: Res<Config>,
    mut bindings: EventWriter<OxrSuggestActionBinding>,
) {
    for binding in config.bindings.clone() {
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
pub fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands, config: Res<Config>) {
    cmds.insert_resource(XrActions::from_config(config.clone(), &instance));
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

        return Self { set, actions };
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug)]
pub enum XrActionType {
    Float,
    Vec2,
    Bool,
    Pose,
}

#[derive(Debug, Clone, Copy)]
pub enum XrActionState {
    Float(XrActionStateFloat),
    Bool(XrActionStateBool),
    Vec2(XrActionStateVec2),
}

impl XrActionState {
    pub fn as_float(&self) -> Option<&XrActionStateFloat> {
        match self {
            XrActionState::Float(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<&XrActionStateBool> {
        match self {
            XrActionState::Bool(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_vec2(&self) -> Option<&XrActionStateVec2> {
        match self {
            XrActionState::Vec2(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_float_mut(&mut self) -> Option<&mut XrActionStateFloat> {
        match self {
            XrActionState::Float(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_bool_mut(&mut self) -> Option<&mut XrActionStateBool> {
        match self {
            XrActionState::Bool(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_vec2_mut(&mut self) -> Option<&mut XrActionStateVec2> {
        match self {
            XrActionState::Vec2(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XrActionStateFloat {
    pub previous_val: f32,
    pub cur_val: f32,
    pub pressed: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XrActionStateVec2 {
    pub previous_val: Vec2,
    pub cur_val: Vec2,
    pub pressed_x: bool,
    pub pressed_y: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XrActionStateBool {
    pub previous_val: bool,
    pub cur_val: bool,
    pub pressed: bool,
}

pub enum XrRawActionState {
    Float(openxr::Action<f32>),
    Vec2(openxr::Action<openxr::Vector2f>),
    Bool(openxr::Action<bool>),
    Pose(openxr::Action<openxr::Posef>),
}
