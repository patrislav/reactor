use bevy::{ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    screens::{
        Screen,
        game_over::{GameOver, GameOverCause},
    },
    theme::widget,
};

use super::Menu;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::GameOver), spawn_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::GameOver).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_menu(mut commands: Commands, game_over: Single<&GameOver>) {
    commands.spawn((
        widget::ui_root("Game Over Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::GameOver),
        children![
            widget::header("game over"),
            grid(vec![
                text_from_cause(game_over.cause),
                "you've generated enough electricity to power"
            ]),
            widget::header(text_from_power(game_over.power_generated)),
            widget::button("Back", go_back_on_click),
        ],
    ));
}

fn grid(content: Vec<&'static str>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Val::Px(10.0),
            column_gap: Val::Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(1, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(
            content
                .into_iter()
                .map(|text| (widget::label(text), Node { ..default() })),
        )),
    )
}

fn go_back_on_click(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn text_from_cause(cause: GameOverCause) -> &'static str {
    match cause {
        GameOverCause::PlayerAbandoned => "thank you for playing!",
        GameOverCause::NotEnoughPower => "you were fired for not meeting quotas",
        GameOverCause::Explosion => {
            "boom! that's the sound your reactor made when it reached its pressure limits"
        }
    }
}

fn text_from_power(power: usize) -> &'static str {
    let texts = [
        "absolutely nothing",
        "a small household for few days",
        "a small household for a few months",
        "a small town for a few days",
        "a small town for a few months",
        "a small town for a few years",
        "a small city for a few months",
        "a small city for a few years",
        "a medium city for a few months",
        "a medium city for a few years",
        "a big city for a few months",
        "a big city for a few years",
        "a big city for a few decades",
        "a whole country for a few years",
        "a whole continent for a few years",
        "the whole world for a few years",
    ];

    for (i, text) in texts.into_iter().enumerate() {
        let base = 10usize;
        if power < base.pow(i as u32) {
            return text;
        }
    }

    "the whole world for a thousand years"
}
