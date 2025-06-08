use bevy::prelude::*;

use crate::theme::palette::BUTTON_TEXT;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_water_container, spawn_steam_container),
    );

    app.add_systems(
        Update,
        (update_steam_score,).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct WaterPerActionMarker;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct TotalSteamMarker;

fn spawn_water_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let root = commands
        .spawn((
            Name::new("Water container"),
            StateScoped(Screen::Gameplay),
            WaterContainer,
            ParticleContainer {
                particle: Particle::Water,
                count: 10,
            },
            Transform::from_xyz(-450., 0.0, 40.),
        ))
        .id();

    commands.spawn((
        Name::new("Title"),
        ChildOf(root),
        Anchor::BottomLeft,
        Text2d::new("water"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(WATER_COLOR.into()),
        Transform::from_xyz(-100.0, 50.0, 0.0),
    ));

    let bg = commands
        .spawn((
            Name::new("Box"),
            ChildOf(root),
            Mesh2d(meshes.add(Rectangle::new(200., 100.))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::from(WATER_COLOR)))),
        ))
        .id();

    let control = commands
        .spawn((
            Name::new("Water control"),
            ChildOf(bg),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    commands.spawn((
        Name::new("Water per action"),
        WaterPerActionMarker,
        ChildOf(control),
        Anchor::Center,
        Text2d::new("3"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(BUTTON_TEXT),
    ));
}

fn spawn_steam_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let root = commands
        .spawn((
            Name::new("Steam container"),
            StateScoped(Screen::Gameplay),
            SteamContainer,
            ParticleContainer {
                particle: Particle::Steam,
                count: 0,
            },
            Transform::from_xyz(-450., 200., 40.),
        ))
        .id();

    commands.spawn((
        Name::new("Title"),
        ChildOf(root),
        Anchor::BottomLeft,
        Text2d::new("steam"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(STEAM_COLOR.into()),
        Transform::from_xyz(-100.0, 50.0, 0.0),
    ));

    let bg = commands
        .spawn((
            Name::new("Box"),
            ChildOf(root),
            Mesh2d(meshes.add(Rectangle::new(200., 100.))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::from(STEAM_COLOR)))),
        ))
        .id();

    commands.spawn((
        Name::new("Steam score"),
        TotalSteamMarker,
        ChildOf(bg),
        Anchor::Center,
        Text2d::new("0"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(BUTTON_TEXT),
    ));
}

fn update_steam_score(
    mut text: Single<&mut Text2d, With<TotalSteamMarker>>,
    container: Single<&ParticleContainer, With<SteamContainer>>,
) {
    text.0 = format!("{}", container.count);
}
