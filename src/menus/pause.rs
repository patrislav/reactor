//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    gameplay::{EnergyContainer, ParticleContainer},
    menus::Menu,
    screens::game_over::{GameOver, GameOverCause},
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Pause),
        children![
            widget::header("game paused"),
            widget::button("continue", close_menu),
            widget::button("settings", open_settings_menu),
            widget::button("abandon game", quit_to_title),
        ],
    ));
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<&ParticleContainer, With<EnergyContainer>>,
) {
    let mut power_generated = 0;
    for container in &query {
        power_generated += container.count;
    }
    commands.trigger(GameOver {
        cause: GameOverCause::PlayerAbandoned,
        power_generated,
    });
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
