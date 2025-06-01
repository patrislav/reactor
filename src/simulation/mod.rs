use bevy::prelude::*;
use types::ReactorCore;

mod computation;
pub mod events;
pub mod schedule;
pub mod types;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(schedule::plugin)
        .add_plugins(events::plugin)
        .add_plugins(types::plugin)
        .add_plugins(computation::plugin);
}

pub fn spawn_reactor_core(mut commands: Commands) {
    info!("Spawned reactor core");
    commands.spawn((
        Name::new("Reactor Core"),
        ReactorCore {
            columns: 7,
            rows: 7,
            ..default()
        },
    ));
}
