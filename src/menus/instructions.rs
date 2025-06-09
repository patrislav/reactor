use bevy::prelude::*;

use crate::theme::{palette::BUTTON_TEXT, widget};

use super::Menu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::HowToPlay), spawn_menu);
    app.add_systems(OnEnter(Menu::HowToPlay2), spawn_menu2);
    app.add_systems(OnEnter(Menu::HowToPlay3), spawn_menu3);
}

fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Instructions Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::HowToPlay),
        children![(
            Node {
                max_width: Val::Px(900.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(50.), Val::Px(50.), Val::Px(50.), Val::Px(50.),),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BUTTON_TEXT),
            children![
                widget::header("how to play"),
                widget::text("Welcome! You've been chosen to operate our nuclear reactor, hopefully you know how. What? You don't? Don't worry, it's not rocket science!"),
                widget::text("In the center, you'll see the layout of the reactor made of circular cells and square control rods."),
                widget::text("Cells contain fuel - either uranium or xenon. Uranium spontaneously releases neutrons and splits into even more if hit by one. Xenon doesn't do anything. That is, until woken up by a neutron!"),
                widget::text("Control rods can be either inserted (100%) or not (0%). Click on them to toggle their insertion. When inserted, they absorb all neutrons."),
                widget::button("next", second_page),
            ]
        )],
    ));
}

fn spawn_menu2(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Instructions Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::HowToPlay2),
        children![(
            Node {
                max_width: Val::Px(900.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(50.), Val::Px(50.), Val::Px(50.), Val::Px(50.),),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BUTTON_TEXT),
            children![
                widget::header("how to play"),
                widget::text("Neutrons flying around would be useless without something to interact with. That something is water."),
                widget::text("When hit by a neutron, a water particle splits into steam particles that can then be turned into energy by our turbines. Don't starve them, we've got people that rely us so we need steady supply of steam!"),
                widget::text("You can add water by clicking on individual cells or by using the 'distribtue' button in the water section on the left. There, you can also control how many particles will be added each time."),
                widget::button("next", third_page),
            ]
        )],
    ));
}

fn spawn_menu3(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Instructions Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::HowToPlay3),
        children![(
            Node {
                max_width: Val::Px(900.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(50.), Val::Px(50.), Val::Px(50.), Val::Px(50.),),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BUTTON_TEXT),
            children![
                widget::header("how to play"),
                widget::text("Now you know the basics, now for some final tips."),
                widget::text("You might think adding a lot of water is a great idea to ramp up steam production. Be careful! Water absorbs neutrons and prevents them from forming chain reactions! Too much of it may also lead to too high pressure."),
                widget::text("Uranium has a small chance to turn into xenon. Without sufficient reactivity, you're going to end up with too much of dead fuel cells and restarting the reaction might prove difficult."),
                widget::text("Control the reactions by balancing insertion levels of control rods and the water amount in cells."),
                widget::text("Good luck!"),
                widget::button("start", close_menu),
            ]
        )],
    ));
}

fn second_page(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::HowToPlay2);
}

fn third_page(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::HowToPlay3);
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
