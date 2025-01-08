pub mod actions;

use actions::{
    attach_set, create_actions, create_input, end_frame_input, spawn_tracking_rig,
    suggest_action_bindings, sync_actions, update_head_transforms, update_inputs,
    update_local_floor_transforms, update_spaces, update_stage, update_view, Config, XrInput,
};
use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::OxrSendActionBindings, action_set_syncing::OxrActionSetSyncSet,
    openxr_session_running, session::OxrSession, spaces::OxrSpaceSyncSet,
};
use bevy_mod_xr::session::{session_available, XrSessionCreated};

pub struct OpenXRPlugin;

impl Plugin for OpenXRPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(XrSessionCreated, spawn_hands);
        app.insert_resource(Config::default());
        app.add_systems(XrSessionCreated, attach_set);
        app.add_systems(
            PreUpdate,
            sync_actions
                .before(OxrActionSetSyncSet)
                .run_if(openxr_session_running),
        );
        app.add_systems(PreUpdate, update_view.after(update_head_transforms));

        //local floor transforms
        app.add_systems(
            PreUpdate,
            update_local_floor_transforms.after(update_head_transforms),
        );
        //spawn tracking rig
        app.add_systems(XrSessionCreated, spawn_tracking_rig);

        //update stage transforms
        //external
        app.add_systems(PreUpdate, update_stage);

        //head view transforms
        //internal
        app.add_systems(
            PreUpdate,
            update_head_transforms
                .in_set(OxrSpaceSyncSet)
                .run_if(openxr_session_running),
        );
        app.add_systems(OxrSendActionBindings, suggest_action_bindings);
        app.add_systems(
            Startup,
            create_actions
                .before(create_input)
                .run_if(session_available),
        ); // app.add_plugins(TrackingUtilitiesPlugin);
           // app.add_systems(
           //     Startup,
           //     create_input.after(create_actions).run_if(session_available),
           // ); // app.add_plugins(TrackingUtilitiesPlugin);
        app.add_systems(
            PostUpdate,
            create_input.run_if(run_if_no_input).after(attach_set),
        );
        app.add_systems(PreUpdate, update_spaces.after(OxrSpaceSyncSet));
        app.add_systems(PreUpdate, update_inputs);
        app.add_systems(PostUpdate, end_frame_input);
        // app.add_systems(PreUpdate, update_right_grip.after(OxrSpaceSyncSet));
    }
}

fn run_if_no_input(input: Option<Res<XrInput>>, session: Option<Res<OxrSession>>) -> bool {
    // true
    input.is_none() && session.is_some()
}
