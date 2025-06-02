use bevy::prelude::*;

mod crosshair;
mod interaction;
mod level;
mod player;
mod reactor_screens;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        player::plugin,
        crosshair::plugin,
        interaction::plugin,
        reactor_screens::plugin,
    ));
}
