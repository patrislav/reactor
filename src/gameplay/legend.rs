use std::time::SystemTimeError;

use bevy::prelude::*;

use crate::screens::Screen;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_layout);

    app.add_observer(add_legend_neutron)
        .add_observer(add_legend_water)
        .add_observer(add_legend_cell)
        .add_observer(add_legend_uranium)
        .add_observer(add_legend_xenon)
        .add_observer(add_legend_control_rod)
        .add_observer(add_legend_steam);
}

fn spawn_layout(mut commands: Commands) {
    let root = commands
        .spawn((
            Name::new("Legend"),
            Transform::from_xyz(500., 250., 50.),
            Visibility::Inherited,
            StateScoped(Screen::Gameplay),
        ))
        .id();

    commands.spawn((
        Name::new("Legend text"),
        Transform::from_xyz(-50.0, 0., 1.),
        ChildOf(root),
        Text2d::new("legend"),
        TextFont {
            font_size: 36.,
            ..default()
        },
        TextColor::WHITE,
    ));

    let gap = 45.;
    let gap2 = 60.;
    let start = -60.;
    let start3 = -110.;

    legend(&mut commands, root, LegendNeutron, 0., start);
    legend(&mut commands, root, LegendWater, 0., start - gap * 1.);
    legend(&mut commands, root, LegendSteam, 0., start - gap * 2.);

    legend(&mut commands, root, LegendCell, 0., start - gap2 * 3.);
    legend(&mut commands, root, LegendUranium, 0., start - gap2 * 4.);
    legend(&mut commands, root, LegendXenon, 0., start - gap2 * 5.);

    legend(
        &mut commands,
        root,
        LegendControlRod,
        0.,
        start3 - gap2 * 6.,
    );
}

fn legend<T: Component>(commands: &mut Commands, root: Entity, comp: T, x: f32, y: f32) {
    commands.spawn((
        comp,
        ChildOf(root),
        Transform::from_xyz(x, y, 0.),
        Visibility::Inherited,
    ));
}

const PARTICLE_X: f32 = -85.;
const PARTICLE_TEXT_X: f32 = -50.;
const FUEL_X: f32 = -75.;
const FUEL_TEXT_X: f32 = -30.;

#[derive(Component)]
struct LegendNeutron;

fn add_legend_neutron(
    trigger: Trigger<OnAdd, LegendNeutron>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(NEUTRON_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(NEUTRON_COLOR))),
        Transform::from_xyz(PARTICLE_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(PARTICLE_TEXT_X, 0., 0.),
        Text2d::new("neutron"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendCell;

fn add_legend_cell(
    trigger: Trigger<OnAdd, LegendCell>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(FUEL_ROD_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(CELL_COLOR))),
        Transform::from_xyz(FUEL_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(FUEL_TEXT_X, 0., 0.),
        Text2d::new("fuel cell"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendUranium;

fn add_legend_uranium(
    trigger: Trigger<OnAdd, LegendUranium>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(FUEL_ROD_RADIUS))),
        MeshMaterial2d(materials.add(URANIUM_COLOR)),
        Transform::from_xyz(FUEL_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(FUEL_TEXT_X, 0., 0.),
        Text2d::new("uranium"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendXenon;

fn add_legend_xenon(
    trigger: Trigger<OnAdd, LegendXenon>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(FUEL_ROD_RADIUS))),
        MeshMaterial2d(materials.add(XENON_COLOR)),
        Transform::from_xyz(FUEL_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(FUEL_TEXT_X, 0., 0.),
        Text2d::new("xenon"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendWater;

fn add_legend_water(
    trigger: Trigger<OnAdd, LegendWater>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(PARTICLE_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(WATER_COLOR))),
        Transform::from_xyz(PARTICLE_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(PARTICLE_TEXT_X, 0., 0.),
        Text2d::new("water"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendSteam;

fn add_legend_steam(
    trigger: Trigger<OnAdd, LegendSteam>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Circle::new(PARTICLE_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(STEAM_COLOR))),
        Transform::from_xyz(PARTICLE_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(PARTICLE_TEXT_X, 0., 0.),
        Text2d::new("steam"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}

#[derive(Component)]
struct LegendControlRod;

fn add_legend_control_rod(
    trigger: Trigger<OnAdd, LegendControlRod>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ChildOf(trigger.target()),
        Mesh2d(meshes.add(Rectangle::from_length(CONTROL_ROD_RADIUS * 1.5))),
        MeshMaterial2d(materials.add(CONTROL_ROD_COLOR_INSERTED)),
        Transform::from_xyz(FUEL_X, 0., 0.),
    ));
    commands.spawn((
        ChildOf(trigger.target()),
        Transform::from_xyz(FUEL_TEXT_X, 0., 0.),
        Text2d::new("control rod"),
        Anchor::CenterLeft,
        TextFont {
            font_size: 18.,
            ..default()
        },
        UseBoldFont,
        TextColor::WHITE,
    ));
}
