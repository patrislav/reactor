use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SkeinPlugin::default(),
            PhysicsPlugins::default(),
        ))
        .run()
}
