use bevy::prelude::*;

use crate::{gameplay::CrtSettings, menus::Menu};

use super::Screen;

#[derive(Reflect, Clone, Copy, Eq, PartialEq)]
pub enum GameOverCause {
    PlayerAbandoned,
    NotEnoughPower,
    Explosion,
}

#[derive(Event, Component, Reflect, Clone, Copy)]
pub struct GameOver {
    pub power_generated: usize,
    pub cause: GameOverCause,
}

pub(super) fn plugin(app: &mut App) {
    app.add_event::<GameOver>();
    app.add_observer(handle_game_over);

    app.add_systems(
        OnEnter(Screen::GameOver),
        (spawn_game_over, open_game_over_menu).chain(),
    );
}

fn handle_game_over(
    trigger: Trigger<GameOver>,
    mut commands: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    next_screen.set(Screen::GameOver);
    commands.spawn((
        Name::new("GameOver"),
        *trigger.event(),
        StateScoped(Screen::GameOver),
    ));
}

fn spawn_game_over(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        StateScoped(Screen::GameOver),
        CrtSettings::default(),
    ));
}

fn open_game_over_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::GameOver);
}
