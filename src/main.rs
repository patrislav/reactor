use bevy::prelude::*;
use reactor::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}
