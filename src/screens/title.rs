//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Title),
        (spawn_main_menu, open_main_menu).chain(),
    );
    app.add_systems(OnExit(Screen::Title), close_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d, StateScoped(Screen::Title)));
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
