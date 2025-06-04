use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use reactor::{asset_tracking, gameplay};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SkeinPlugin::default(),
            PhysicsPlugins::default(),
            asset_tracking::plugin,
            gameplay::plugin,
        ))
        .run()
}
