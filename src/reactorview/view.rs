use std::any::TypeId;

use bevy::{
    color::palettes::css,
    platform::collections::HashMap,
    prelude::*,
    reflect::{TypeRegistration, Typed},
    render::view::RenderLayers,
};

use crate::{
    screens::Screen,
    simulation::types::{
        ControlRod, Reactivity, ReactorCell, ReactorCore, ReactorCoreReady, Temperature,
    },
};

use super::edges::DisplayReactorEdge;

pub fn plugin<const N: usize>(app: &mut App) {
    app.init_state::<DisplayMode<N>>();

    app.add_plugins(super::edges::plugin::<N>);
    app.add_observer(on_add_reactor_core_ready::<N>);

    app.add_systems(
        OnEnter(DisplayMode::<N>::Control),
        prepare_control_layout::<N>,
    );
    app.add_systems(
        OnEnter(DisplayMode::<N>::Temperature),
        prepare_temperature_layout::<N>,
    );
    app.add_systems(
        OnEnter(DisplayMode::<N>::Cooling),
        prepare_cooling_layout::<N>,
    );
    app.add_observer(switch_display_mode::<N>);

    app.add_systems(
        Update,
        (
            sync_display_with_simulation::<N, Reactivity>,
            sync_display_with_simulation::<N, Temperature>,
            sync_display_with_simulation::<N, ControlRod>,
        ),
    );
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
struct ReactorViewLink<const N: usize>(Entity);

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum DisplayMode<const N: usize> {
    #[default]
    Loading,
    Control,
    Cooling,
    Temperature,
    Pressure,
}

impl<const N: usize> DisplayMode<N> {
    pub fn edge_visible(&self, edge: &DisplayReactorEdge<N>) -> bool {
        match *self {
            Self::Temperature => true,
            Self::Cooling => edge.valve.is_some(),
            _ => false,
        }
    }
}

#[derive(Component, Clone, Default, Reflect)]
pub struct Grid<const N: usize> {
    pub simulation_to_display_cells: HashMap<Entity, Entity>,
}

#[derive(Component, Clone, Copy, Reflect)]
pub struct ReactorCellLink<const N: usize>(pub Entity);

#[derive(Component, Clone, Default, Reflect)]
struct ParameterLinks<const N: usize>(HashMap<TypeId, Entity>);

impl<const N: usize> ParameterLinks<N> {
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

pub const CELL_SIZE: f32 = 80.;
pub const EDGE_WIDTH: f32 = 15.;
pub const FONT_SIZE: f32 = 15.;
pub const TEXT_PADDING: f32 = 5.;

fn prepare_control_layout<const N: usize>(
    mut commands: Commands,
    cells: Query<&ParameterLinks<N>>,
) -> Result {
    for param_links in &cells {
        hide_all_params(&mut commands, param_links);

        show_param::<N, ControlRod>(&mut commands, param_links, Transform::default());
    }
    Ok(())
}

fn prepare_temperature_layout<const N: usize>(
    mut commands: Commands,
    cells: Query<&ParameterLinks<N>>,
) -> Result {
    for param_links in &cells {
        hide_all_params(&mut commands, param_links);

        show_param::<N, Temperature>(&mut commands, param_links, Transform::default());
    }
    Ok(())
}

fn prepare_cooling_layout<const N: usize>(
    mut commands: Commands,
    cells: Query<&ParameterLinks<N>>,
) -> Result {
    for param_links in &cells {
        hide_all_params(&mut commands, param_links);
    }
    Ok(())
}

fn hide_all_params<const N: usize>(commands: &mut Commands, param_links: &ParameterLinks<N>) {
    hide_param::<N, Reactivity>(commands, param_links);
    hide_param::<N, Temperature>(commands, param_links);
    hide_param::<N, ControlRod>(commands, param_links);
}

fn hide_param<const N: usize, T>(commands: &mut Commands, param_links: &ParameterLinks<N>)
where
    T: Typed,
{
    if let Some(entity) = param_links.get::<T>() {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}

fn show_param<const N: usize, T>(
    commands: &mut Commands,
    param_links: &ParameterLinks<N>,
    bundle: impl Bundle,
) where
    T: Typed,
{
    if let Some(entity) = param_links.get::<T>() {
        commands
            .entity(entity)
            .insert(Visibility::Inherited)
            .insert(bundle);
    }
}

pub fn on_add_reactor_core_ready<const N: usize>(
    trigger: Trigger<OnAdd, ReactorCoreReady>,
    mut commands: Commands,
    screen: Res<State<Screen>>,
    mut display_mode: ResMut<NextState<DisplayMode<N>>>,
    cores: Query<&ReactorCore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    let core = cores.get(trigger.target())?;

    let render_layer = RenderLayers::layer(N);
    let mut grid = Grid::<N>::default();
    let grid_entity = commands
        .spawn((
            Name::new(format!("ReactorView {}", N)),
            Visibility::default(),
            Transform::from_xyz(0., 0., 0.),
            StateScoped(match *screen.get() {
                Screen::SimulationTesting => Screen::SimulationTesting,
                _ => Screen::Gameplay,
            }),
            render_layer.clone(),
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
                ReactorCellLink::<N>(entity),
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(Color::from(css::GRAY))),
                Pickable::default(),
                render_layer.clone(),
            ))
            .id();
        info!("Spawned a display cell: {}", cell);
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
                render_layer.clone(),
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
                render_layer.clone(),
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
                render_layer.clone(),
            ))
            .id();
        commands.entity(cell).insert(
            ParameterLinks::<N>::default()
                .insert::<Temperature>(temp_text)
                .insert::<Reactivity>(react_text)
                .insert::<ControlRod>(rod_text),
        );
    }

    display_mode.set(DisplayMode::Temperature);
    commands.entity(grid_entity).insert(grid);

    Ok(())
}

#[derive(Event, Copy, Clone, Default, Reflect)]
pub struct CycleDisplayMode(pub usize);

fn switch_display_mode<const N: usize>(
    trigger: Trigger<CycleDisplayMode>,
    current_mode: Res<State<DisplayMode<N>>>,
    mut next_mode: ResMut<NextState<DisplayMode<N>>>,
) {
    if trigger.0 == N {
        next_mode.set(match current_mode.get() {
            DisplayMode::Control => DisplayMode::Temperature,
            DisplayMode::Temperature => DisplayMode::Cooling,
            _ => DisplayMode::Control,
        });
    }
}

fn sync_display_with_simulation<const N: usize, T>(
    mut commands: Commands,
    grids: Query<&Grid<N>>,
    display_links: Query<&ParameterLinks<N>>,
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
