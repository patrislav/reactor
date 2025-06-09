use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::theme::palette::BUTTON_TEXT;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            spawn_water_container,
            spawn_steam_container,
            spawn_power_container,
        ),
    );

    app.add_systems(
        Update,
        (
            update_water_flow,
            update_steam_score,
            update_power_score,
            update_power_demand,
            update_colors,
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct WaterPerActionMarker;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct TotalSteamMarker;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct TotalEnergyMarker;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
struct PowerDemandMarker;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct ParticleContainerColor(pub Color);

fn spawn_water_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let button_mesh = meshes.add(Rectangle::from_length(26.));
    let distribute_mesh = meshes.add(Rectangle::from_size(Vec2::new(100., 26.)));
    let button_material = materials.add(Color::WHITE);

    let root = commands
        .spawn((
            Name::new("Water container"),
            StateScoped(Screen::Gameplay),
            WaterContainer,
            ParticleContainer {
                particle: Particle::Water(false),
                count: 10,
            },
            ParticleContainerColor(Color::from(WATER_COLOR)),
            Transform::from_xyz(-500., -200., 40.),
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
        ChildOf(control),
        WaterPerActionMarker,
        Transform::from_xyz(0., 25., 1.),
        Anchor::Center,
        Text2d::new("3"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(BUTTON_TEXT),
    ));
    commands
        .spawn((
            Name::new("Minus"),
            ChildOf(control),
            Mesh2d(button_mesh.clone()),
            MeshMaterial2d(button_material.clone()),
            Transform::from_xyz(-50., 25., 1.),
            Pickable::default(),
            PlaysClickSound,
            PlaysHoverSound,
            children![(
                Name::new("Minus label"),
                Text2d::new("<"),
                TextFont {
                    font_size: 34.,
                    ..default()
                },
                UseBoldFont,
                TextColor(WATER_COLOR.into()),
                Transform::from_xyz(0., 2., 1.),
                Pickable::IGNORE,
            )],
        ))
        .observe(on_click_water_control_decrease);
    commands
        .spawn((
            Name::new("Plus"),
            ChildOf(control),
            Mesh2d(button_mesh.clone()),
            MeshMaterial2d(button_material.clone()),
            Transform::from_xyz(50., 25., 1.),
            Pickable::default(),
            PlaysClickSound,
            PlaysHoverSound,
            children![(
                Name::new("Plus label"),
                Text2d::new(">"),
                TextFont {
                    font_size: 34.,
                    ..default()
                },
                UseBoldFont,
                TextColor(WATER_COLOR.into()),
                Transform::from_xyz(0., 2., 1.),
                Pickable::IGNORE,
            )],
        ))
        .observe(on_click_water_control_increase);
    commands
        .spawn((
            Name::new("Distribute"),
            ChildOf(control),
            Mesh2d(distribute_mesh.clone()),
            MeshMaterial2d(button_material.clone()),
            Transform::from_xyz(0., -25., 1.),
            Pickable::default(),
            PlaysClickSound,
            PlaysHoverSound,
            children![(
                Name::new("Distribute label"),
                Text2d::new("distribute"),
                TextFont {
                    font_size: 18.,
                    ..default()
                },
                UseBoldFont,
                TextColor(BUTTON_TEXT),
                Transform::from_xyz(0., 0., 1.),
                Pickable::IGNORE,
            )],
        ))
        .observe(on_click_water_distribute);
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
            ParticleContainerColor(Color::from(STEAM_COLOR)),
            Transform::from_xyz(-500., 0., 40.),
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

fn spawn_power_container(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let root = commands
        .spawn((
            Name::new("Power container"),
            StateScoped(Screen::Gameplay),
            EnergyContainer,
            TicksWithoutPower::default(),
            ParticleContainer {
                particle: Particle::Energy,
                count: 0,
            },
            ParticleContainerColor(URANIUM_COLOR),
            Transform::from_xyz(-500., 200., 40.),
        ))
        .id();

    commands.spawn((
        Name::new("Title"),
        ChildOf(root),
        Anchor::BottomLeft,
        Text2d::new("power"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(URANIUM_COLOR),
        Transform::from_xyz(-100.0, 50.0, 0.0),
    ));

    commands.spawn((
        Name::new("Box"),
        ChildOf(root),
        Mesh2d(meshes.add(Rectangle::new(200., 100.))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(URANIUM_COLOR))),
        children![
            (
                Name::new("Power score label"),
                Anchor::TopLeft,
                Transform::from_xyz(-96., 46., 0.0),
                Text2d::new("generated:"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                UseBoldFont,
                TextColor(BUTTON_TEXT),
            ),
            (
                Name::new("Power score"),
                TotalEnergyMarker,
                Anchor::TopRight,
                Transform::from_xyz(96., 46., 0.0),
                Text2d::new("0"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                UseBoldFont,
                TextColor(BUTTON_TEXT),
            ),
            (
                Name::new("Power demand label"),
                Anchor::BottomLeft,
                Transform::from_xyz(-96., -46., 0.0),
                Text2d::new("demand:"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                UseBoldFont,
                TextColor(BUTTON_TEXT),
            ),
            (
                Name::new("Power demand"),
                PowerDemandMarker,
                Anchor::BottomRight,
                Transform::from_xyz(96., -46., 0.0),
                Text2d::new("0"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                UseBoldFont,
                TextColor(BUTTON_TEXT),
            )
        ],
    ));
}

fn update_water_flow(
    mut text: Single<&mut Text2d, With<WaterPerActionMarker>>,
    flow: Single<&WaterFlow, With<WaterContainer>>,
) {
    text.0 = format!("{}", flow.get());
}

fn update_steam_score(
    mut text: Single<&mut Text2d, With<TotalSteamMarker>>,
    container: Single<&ParticleContainer, With<SteamContainer>>,
) {
    text.0 = format!("{}", container.count);
}

fn update_power_score(
    mut text: Single<&mut Text2d, With<TotalEnergyMarker>>,
    container: Single<&ParticleContainer, With<EnergyContainer>>,
) {
    text.0 = format_power(container.count as u64); // TODO: change the container.count type
}

fn update_power_demand(
    mut text: Single<&mut Text2d, With<PowerDemandMarker>>,
    demand: Single<&PowerDemand, With<EnergyContainer>>,
) {
    text.0 = format_power(demand.0 as u64); // TODO: change the demand.0 type
}

fn format_power(power: u64) -> String {
    if power > 10_000_000_000 {
        format!("{} B", power / 1_000_000_000)
    } else if power > 10_000_000 {
        format!("{} M", power / 1_000_000)
    } else if power > 10_000 {
        format!("{} K", power / 1_000)
    } else {
        format!("{}", power)
    }
}

fn update_colors(
    query: Query<(&ParticleContainerColor, &Children), Changed<ParticleContainerColor>>,
    mut text_colors: Query<&mut TextColor>,
    mesh_materials: Query<&mut MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (color, children) in &query {
        for entity in children {
            if let Ok(mut text_color) = text_colors.get_mut(*entity) {
                text_color.0 = color.0;
            } else if let Ok(mesh_material) = mesh_materials.get(*entity) {
                if let Some(material) = materials.get_mut(mesh_material.id()) {
                    material.color = color.0;
                }
            }
        }
    }
}

fn on_click_water_control_decrease(_: Trigger<Pointer<Click>>, query: Single<&mut WaterFlow>) {
    query.into_inner().decrease();
}

fn on_click_water_control_increase(_: Trigger<Pointer<Click>>, query: Single<&mut WaterFlow>) {
    query.into_inner().increase();
}

fn on_click_water_distribute(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<Entity, With<Cell>>,
) {
    let mut rng = rand::rng();
    let mut cells: Vec<_> = query.into_iter().collect();
    cells.shuffle(&mut rng);

    for cell in cells {
        commands.trigger_targets(FlowWaterParticlesIntoCell, cell);
    }
}
