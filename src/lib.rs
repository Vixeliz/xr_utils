use bevy::prelude::*;

mod grab;
mod gravity_grab;
mod input;

pub mod prelude {
    pub use crate::grab::{Grabbable, Holding};
    pub use crate::gravity_grab::GravityGrabbing;
    pub use crate::XrUtilsPlugin;
}

pub struct XrUtilsPlugin {}
impl Plugin for XrUtilsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, gravity_grab::outlines.before(gravity_grab::gesture));
        app.add_systems(Update, gravity_grab::gesture);
        app.add_systems(Update, grab::grab);
    }
}
