use bevy::prelude::*;
use input::{actions::XrActionType, OpenXRPlugin};
use prelude::{handle_transform_events, SnapToPosition, SnapToRotation, XrAction};

mod grab;
mod gravity_grab;
mod input;
mod movement;

pub mod prelude {
    pub use crate::grab::{Grabbable, Holding};
    pub use crate::gravity_grab::GravityGrabbing;
    pub use crate::input::actions::{
        HeadXRSpace, XrAction, XrActionType, XrInput, XrTrackedLocalFloor, XrTrackedSpace,
        XrTrackedStage, XrTrackedView,
    };
    pub use crate::movement::*;
    pub use crate::XrUtilsPlugin;
}

#[derive(Resource)]
pub struct XrUtilsConfig {
    /// Only supports floats or bools also needs a pose
    gravity_grab_action_names: Vec<(XrAction, XrAction)>,
    /// Only supports floats or bools also needs a pose
    grab_action_names: Vec<(XrAction, XrAction)>,
}

impl Default for XrUtilsConfig {
    fn default() -> Self {
        Self {
            gravity_grab_action_names: vec![
                (
                    XrAction::from_string(&"right_squeeze".to_string(), &XrActionType::Float),
                    XrAction::from_string(&"right_pose".to_string(), &XrActionType::Pose),
                ),
                (
                    XrAction::from_string(&"left_squeeze".to_string(), &XrActionType::Float),
                    XrAction::from_string(&"left_pose".to_string(), &XrActionType::Pose),
                ), // ("left_squeeze".to_string(), XrActionType::Float),
            ],
            grab_action_names: vec![
                (
                    XrAction::from_string(&"right_squeeze".to_string(), &XrActionType::Float),
                    XrAction::from_string(&"right_pose".to_string(), &XrActionType::Pose),
                ),
                (
                    XrAction::from_string(&"left_squeeze".to_string(), &XrActionType::Float),
                    XrAction::from_string(&"left_pose".to_string(), &XrActionType::Pose),
                ),
                // ("left_squeeze".to_string(), XrActionType::Float),
            ],
        }
    }
}

pub struct XrUtilsPlugin;
impl Plugin for XrUtilsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(XrUtilsConfig::default());
        app.add_event::<SnapToRotation>();
        app.add_event::<SnapToPosition>();
        app.add_systems(PostUpdate, handle_transform_events);
        app.add_plugins(OpenXRPlugin);
        app.add_systems(Update, gravity_grab::outlines.before(gravity_grab::gesture));
        app.add_systems(Update, gravity_grab::gesture);
        app.add_systems(Update, gravity_grab::gravity_grabbing);
        app.add_systems(Update, grab::grab);
    }
}
