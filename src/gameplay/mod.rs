use avian3d::prelude::PhysicsLayer;
use bevy::prelude::*;

mod crosshair;
mod crt;
mod game_object;
mod interaction;
mod level;
mod player;
mod reactor_screens;

#[derive(Default, Copy, Clone, Reflect, PhysicsLayer)]
pub enum GameLayer {
    #[default]
    Default,
    Environment,
    Player,
    Interactable,
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        player::plugin,
        crosshair::plugin,
        interaction::plugin,
        reactor_screens::plugin,
        crt::plugin,
    ));
}
