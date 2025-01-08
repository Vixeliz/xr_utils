use bevy::prelude::*;
use input::{actions::XrActionType, OpenXRPlugin};

mod grab;
mod gravity_grab;
mod input;

pub mod prelude {
    pub use crate::grab::{Grabbable, Holding};
    pub use crate::gravity_grab::GravityGrabbing;
    pub use crate::input::actions::{
        XrAction, XrActionType, XrInput, XrTrackedLocalFloor, XrTrackedSpace, XrTrackedStage,
        XrTrackedView,
    };
    pub use crate::XrUtilsPlugin;
}

#[derive(Resource)]
pub struct XrUtilsConfig {
    /// Only supports floats or bools
    gravity_grab_action_names: Vec<(String, XrActionType)>,
    /// Only supports floats or bools
    grab_action_names: Vec<(String, XrActionType)>,
}

impl Default for XrUtilsConfig {
    fn default() -> Self {
        Self {
            gravity_grab_action_names: vec![
                ("right_squeeze".to_string(), XrActionType::Float),
                // ("left_squeeze".to_string(), XrActionType::Float),
            ],
            grab_action_names: vec![
                ("right_squeeze".to_string(), XrActionType::Float),
                // ("left_squeeze".to_string(), XrActionType::Float),
            ],
        }
    }
}

pub struct XrUtilsPlugin;
impl Plugin for XrUtilsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(XrUtilsConfig::default());
        app.add_plugins(OpenXRPlugin);
        app.add_systems(Update, gravity_grab::outlines.before(gravity_grab::gesture));
        app.add_systems(Update, gravity_grab::gesture);
        app.add_systems(Update, gravity_grab::gravity_grabbing);
        app.add_systems(Update, grab::grab);
    }
}
