use std::any::TypeId;

use bevy::{
    color::palettes::css,
    platform::collections::HashMap,
    prelude::*,
    reflect::{TypeRegistration, Typed},
};

use crate::simulation::{events::MoveControlRod, types::*};

mod edges;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<DisplayMode>();

    app.add_plugins(edges::plugin);
    app.add_observer(on_add_reactor_core_ready);

    app.add_systems(OnEnter(DisplayMode::Control), prepare_control_layout);
    app.add_systems(
        OnEnter(DisplayMode::Temperature),
        prepare_temperature_layout,
    );
    app.add_systems(Update, switch_display_mode);

    app.add_systems(
        Update,
        (
            sync_display_with_simulation::<Reactivity>,
            sync_display_with_simulation::<Temperature>,
            sync_display_with_simulation::<ControlRod>,
        ),
    );

    app.add_systems(Update, (update_displayed_reactivity,).chain());
    app.add_systems(Update, (update_displayed_temperature,).chain());
    app.add_systems(Update, (update_displayed_control_rod,).chain());
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum DisplayMode {
    #[default]
    Loading,
    Control,
    Cooling,
    Temperature,
    Pressure,
}

impl DisplayMode {
    fn edges_visible(&self) -> bool {
        *self == Self::Temperature
    }

    fn coolant_channels_visible(&self) -> bool {
        *self == Self::Cooling
    }
}

#[derive(Component, Clone, Default, Reflect)]
struct Grid {
    simulation_to_display_cells: HashMap<Entity, Entity>,
}

#[derive(Component, Clone, Copy, Reflect)]
struct ReactorCellLink(Entity);

#[derive(Component, Clone, Default, Reflect)]
struct ParameterLinks(HashMap<TypeId, Entity>);

impl ParameterLinks {
    fn get<T>(&self) -> Option<Entity>
    where
        T: Typed,
    {
        self.0.get(&TypeRegistration::of::<T>().type_id()).copied()
    }

    fn insert<T>(mut self, entity: Entity) -> Self
    where
        T: Typed,
    {
        self.0.insert(TypeRegistration::of::<T>().type_id(), entity);
        self
    }
}

#[derive(Component)]
struct ModeMarkerText;

pub const CELL_SIZE: f32 = 100.;
pub const EDGE_WIDTH: f32 = 15.;
pub const FONT_SIZE: f32 = 15.;
pub const TEXT_PADDING: f32 = 5.;

fn prepare_control_layout(mut commands: Commands, cells: Query<&ParameterLinks>) -> Result {
    for param_links in &cells {
        hide_param::<Reactivity>(&mut commands, param_links);
        hide_param::<Temperature>(&mut commands, param_links);

        show_param::<ControlRod>(&mut commands, param_links, Transform::default());
    }
    Ok(())
}

fn prepare_temperature_layout(mut commands: Commands, cells: Query<&ParameterLinks>) -> Result {
    for param_links in &cells {
        hide_param::<Reactivity>(&mut commands, param_links);
        hide_param::<ControlRod>(&mut commands, param_links);

        show_param::<Temperature>(&mut commands, param_links, Transform::default());
    }
    Ok(())
}

fn hide_param<T>(commands: &mut Commands, param_links: &ParameterLinks)
where
    T: Typed,
{
    if let Some(entity) = param_links.get::<T>() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}

fn show_param<T>(commands: &mut Commands, param_links: &ParameterLinks, bundle: impl Bundle)
where
    T: Typed,
{
    if let Some(entity) = param_links.get::<T>() {
        commands
            .entity(entity)
            .insert(Visibility::Inherited)
            .insert(bundle);
    }
}

pub fn on_add_reactor_core_ready(
    _trigger: Trigger<OnAdd, ReactorCoreReady>,
    mut commands: Commands,
    mut display_mode: ResMut<NextState<DisplayMode>>,
    core: Single<&ReactorCore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    let mut grid = Grid::default();
    let grid_entity = commands
        .spawn((
            Name::new("Grid"),
            Visibility::default(),
            Transform::from_xyz(0., 0., 0.),
        ))
        .id();

    let padding = 10.;
    let container_size = CELL_SIZE + padding * 2.;
    let mesh = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));

    for (pos, entity) in core.iter_cells() {
        let cell = commands
            .spawn((
                ChildOf(grid_entity),
                Transform::from_xyz(
                    (pos.x as f32) * container_size,
                    (pos.y as f32) * container_size,
                    8.0,
                ),
                ReactorCellLink(entity),
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(Color::from(css::GRAY))),
                Pickable::default(),
            ))
            .observe(on_cell_click)
            .id();
        grid.simulation_to_display_cells.insert(entity, cell);

        let rod_text = commands
            .spawn((
                ChildOf(cell),
                Visibility::Hidden,
                Text2d::new(""),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(css::BLACK.into()),
            ))
            .id();
        let react_text = commands
            .spawn((
                ChildOf(cell),
                Visibility::Hidden,
                Text2d::new(""),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(css::BLACK.into()),
            ))
            .id();
        let temp_text = commands
            .spawn((
                ChildOf(cell),
                Visibility::Hidden,
                Text2d::new(""),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(css::BLACK.into()),
            ))
            .id();
        commands.entity(cell).insert(
            ParameterLinks::default()
                .insert::<Temperature>(temp_text)
                .insert::<Reactivity>(react_text)
                .insert::<ControlRod>(rod_text),
        );
    }

    display_mode.set(DisplayMode::Control);
    commands.entity(grid_entity).insert(grid);
    Ok(())
}

fn update_displayed_reactivity(
    mut commands: Commands,
    query: Query<(Entity, &Reactivity), (With<Text2d>, Changed<Reactivity>)>,
) {
    for (entity, reactivity) in &query {
        let text = format!("{:.1}", reactivity.0);
        commands.entity(entity).insert(Text2d::new(text));
    }
}

fn update_displayed_temperature(
    mut commands: Commands,
    query: Query<(Entity, &Temperature), (With<Text2d>, Changed<Temperature>)>,
) {
    for (entity, temp) in &query {
        let text = format!("{:.1}°", temp.0);
        commands.entity(entity).insert(Text2d::new(text));
    }
}

fn update_displayed_control_rod(
    mut commands: Commands,
    query: Query<(Entity, &ControlRod), (With<Text2d>, Changed<ControlRod>)>,
) {
    for (entity, rod) in &query {
        let text = format!("{:.0}%", rod.0 * 100.);
        commands.entity(entity).insert(Text2d::new(text));
    }
}

fn sync_display_with_simulation<T>(
    mut commands: Commands,
    grids: Query<&Grid>,
    display_links: Query<&ParameterLinks>,
    simulation_values: Query<(Entity, &T), (With<ReactorCell>, Changed<T>)>,
) where
    T: Component + Clone + Reflect,
{
    for grid in &grids {
        for (simulation_cell, component) in &simulation_values {
            let Some(display_cell_entity) = grid.simulation_to_display_cells.get(&simulation_cell)
            else {
                warn!("No display cell entity found");
                continue;
            };

            let Ok(links) = display_links.get(*display_cell_entity) else {
                warn!("No ParameterLinks found");
                continue;
            };

            let Some(link_entity) = links.0.get(&component.type_id()) else {
                continue;
            };

            commands.entity(*link_entity).insert(component.clone());
        }
    }
}

fn on_cell_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    display_mode: Res<State<DisplayMode>>,
    buttons: Query<&ReactorCellLink>,
) -> Result {
    let cell = buttons.get(trigger.target())?.0;

    match display_mode.get() {
        DisplayMode::Control => {
            let val = match trigger.event().button {
                PointerButton::Primary => -0.1,
                PointerButton::Secondary => 0.1,
                _ => return Ok(()),
            };
            commands.trigger_targets(MoveControlRod(val), cell);
        }
        _ => {}
    }
    Ok(())
}

fn switch_display_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<DisplayMode>>,
    mut next_mode: ResMut<NextState<DisplayMode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_mode.set(match current_mode.get() {
            DisplayMode::Control => DisplayMode::Temperature,
            _ => DisplayMode::Control,
        });
    }
}
