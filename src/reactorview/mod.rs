use bevy::{color::palettes::css, platform::collections::HashMap, prelude::*};

use crate::simulation::{events::MoveControlRod, types::*};

pub(super) fn plugin(app: &mut App) {
    //app.add_systems(Startup, spawn_layout);
    app.add_observer(on_add_reactor_core_ready);
    app.add_systems(
        Update,
        (
            sync_display_with_simulation::<Reactivity, ReactivityLink>,
            update_displayed_reactivity,
        )
            .chain(),
    );
    app.add_systems(
        Update,
        (
            sync_display_with_simulation::<Temperature, TemperatureLink>,
            update_displayed_temperature,
        )
            .chain(),
    );
    app.add_systems(
        Update,
        (
            sync_display_with_simulation::<ControlRod, ControlRodLink>,
            update_displayed_control_rod,
        )
            .chain(),
    );
}

#[derive(Component, Clone, Default, Reflect)]
struct Grid {
    simulation_to_display_cells: HashMap<Entity, Entity>,
}

#[derive(Component, Clone, Copy, Reflect)]
struct ReactorCellLink(Entity);

#[derive(Component, Clone, Copy, Reflect)]
struct ReactivityLink(Entity);

impl From<ReactivityLink> for Entity {
    fn from(value: ReactivityLink) -> Self {
        value.0
    }
}

#[derive(Component, Clone, Copy, Reflect)]
struct TemperatureLink(Entity);

impl From<TemperatureLink> for Entity {
    fn from(value: TemperatureLink) -> Self {
        value.0
    }
}

#[derive(Component, Clone, Copy, Reflect)]
struct ControlRodLink(Entity);

impl From<ControlRodLink> for Entity {
    fn from(value: ControlRodLink) -> Self {
        value.0
    }
}

#[derive(Component)]
struct ModeMarkerText;

pub fn on_add_reactor_core_ready(
    trigger: Trigger<OnAdd, ReactorCoreReady>,
    mut commands: Commands,
    cores: Query<&ReactorCore>,
) -> Result {
    let core = cores.get(trigger.target())?;

    let root = commands
        .spawn((
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                grid_template_columns: vec![GridTrack::auto()],
                grid_template_rows: vec![GridTrack::auto(), GridTrack::flex(1.0)],
                ..default()
            },
            children![(
                Node {
                    grid_column: GridPlacement::span(1),
                    padding: UiRect::all(Val::Px(6.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(css::WHITE.into()),
                children![(Text::new("Current mode: default"), TextColor::BLACK,)]
            ),],
        ))
        .id();

    let mut grid = Grid::default();
    let grid_entity = commands
        .spawn((
            ChildOf(root),
            Node {
                display: Display::Grid,
                justify_self: JustifySelf::End,
                height: Val::Percent(100.0),
                aspect_ratio: Some((core.rows as f32) / (core.columns as f32)),
                grid_template_rows: RepeatedGridTrack::flex((core.rows) as u16, 1.0),
                grid_template_columns: RepeatedGridTrack::flex((core.columns) as u16, 1.0),
                grid_auto_flow: GridAutoFlow::Column,
                ..default()
            },
        ))
        .id();

    for (pos, maybe_entity) in core.iter_all_positions_with_cells() {
        if let Some(entity) = maybe_entity {
            let container = commands
                .spawn((
                    Name::new("Cell"),
                    ChildOf(grid_entity),
                    Node {
                        padding: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                ))
                .id();
            let cell = commands
                .spawn((
                    Button,
                    ChildOf(container),
                    ReactorCellLink(entity),
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(css::DARK_GOLDENROD.into()),
                    BackgroundColor(css::GRAY.into()),
                ))
                .observe(on_cell_click)
                .id();
            grid.simulation_to_display_cells.insert(entity, cell);

            let rod_text = commands
                .spawn((ChildOf(cell), Text::new(""), TextColor(css::BLACK.into())))
                .id();
            let react_text = commands
                .spawn((ChildOf(cell), Text::new(""), TextColor(css::BLACK.into())))
                .id();
            let temp_text = commands
                .spawn((ChildOf(cell), Text::new(""), TextColor(css::BLACK.into())))
                .id();
            commands.entity(cell).insert((
                TemperatureLink(temp_text),
                ReactivityLink(react_text),
                ControlRodLink(rod_text),
            ));
        } else {
            commands.spawn((
                Name::new("Spacer"),
                ChildOf(grid_entity),
                Node { ..default() },
            ));
        }
    }

    commands.entity(grid_entity).insert(grid);

    Ok(())
}

fn update_displayed_reactivity(
    mut commands: Commands,
    query: Query<(Entity, &Reactivity), (With<Text>, Changed<Reactivity>)>,
) {
    for (entity, reactivity) in &query {
        let text = format!("R: {:.1}", reactivity.0);
        commands.entity(entity).insert(Text::new(text));
    }
}

fn update_displayed_temperature(
    mut commands: Commands,
    query: Query<(Entity, &Temperature), (With<Text>, Changed<Temperature>)>,
) {
    for (entity, temp) in &query {
        let text = format!("T: {:.1}°", temp.0);
        commands.entity(entity).insert(Text::new(text));
    }
}

fn update_displayed_control_rod(
    mut commands: Commands,
    query: Query<(Entity, &ControlRod), (With<Text>, Changed<ControlRod>)>,
) {
    for (entity, rod) in &query {
        let text = format!("C: {:.0}%", rod.0 * 100.);
        commands.entity(entity).insert(Text::new(text));
    }
}

fn sync_display_with_simulation<T, L>(
    mut commands: Commands,
    grids: Query<&Grid>,
    display_links: Query<&L>,
    simulation_values: Query<(Entity, &T), (With<ReactorCell>, Changed<T>)>,
) where
    T: Component + Clone,
    L: Component + Clone + Into<Entity>,
{
    for grid in &grids {
        for (simulation_cell, component) in &simulation_values {
            let Some(display_cell_entity) = grid.simulation_to_display_cells.get(&simulation_cell)
            else {
                warn!("No display cell entity found");
                continue;
            };

            let Ok(link) = display_links.get(*display_cell_entity) else {
                warn!("No link found");
                continue;
            };

            commands
                .entity(link.clone().into())
                .insert(component.clone());
        }
    }
}

fn on_cell_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    buttons: Query<&ReactorCellLink>,
) -> Result {
    let val = match trigger.event().button {
        PointerButton::Primary => -0.1,
        PointerButton::Secondary => 0.1,
        _ => return Ok(()),
    };
    let cell = buttons.get(trigger.target())?.0;
    commands.trigger_targets(MoveControlRod(val), cell);
    Ok(())
}
