use bevy::{platform::collections::HashMap, prelude::*};

use super::events;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Temperature>()
        .register_type::<CoolantLevel>()
        .register_type::<Reactivity>()
        .register_type::<LocalReactivity>()
        .register_type::<ControlRod>()
        .register_type::<ReactorCell>()
        .register_type::<ReactorEdge>()
        .init_resource::<SimulationConfig>()
        .add_observer(on_add_reactor_core);
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct SimulationConfig {
    pub base_reactivity: f32,
    pub void_reactivity_boost: f32,
    pub reactivity_neighbor_coupling_factor: f32,
    pub heat_generation_factor: f32,

    pub coolant_temperature: f32,
    pub coolant_efficiency: f32,
    pub temperature_passive_decay_rate: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            base_reactivity: 1.0,
            void_reactivity_boost: 0.5,
            reactivity_neighbor_coupling_factor: 0.1,
            heat_generation_factor: 1.0,
            coolant_temperature: 25.0,
            coolant_efficiency: 0.1,
            temperature_passive_decay_rate: 0.01,
        }
    }
}

/// Temperature in degrees Celsius.
#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Temperature(pub f32);

impl Default for Temperature {
    fn default() -> Self {
        Self(25.0)
    }
}

/// How full the cell is with water, 0.0-1.0.
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct CoolantLevel(pub f32);

/// Coolant flow through the cell per tick, in CoolantLevel units.
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct CoolantFlow(pub f32);

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct LocalReactivity(pub f32);

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Reactivity(pub f32);

/// Control rod insertion: 0.0 - fully raised, 1.0 - fully insered.
#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ControlRod(pub f32);

impl Default for ControlRod {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Default, Reflect, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn neighbours(&self) -> [Self; 4] {
        [
            Self::new(self.x - 1, self.y - 1),
            Self::new(self.x - 1, self.y + 1),
            Self::new(self.x + 1, self.y - 1),
            Self::new(self.x + 1, self.y + 1),
        ]
    }

    pub fn is_valid(&self) -> bool {
        (self.x & 1) + (self.y & 1) == 1
    }
}

#[derive(Component, Clone, Copy, Default, Reflect, Eq, PartialEq, Hash)]
#[reflect(Component)]
#[require(
    ControlRod,
    LocalReactivity,
    Reactivity,
    Temperature,
    CoolantLevel,
    CoolantFlow
)]
pub struct ReactorCell(pub Position);

#[derive(Clone, Copy, Default, Reflect, Eq, PartialEq)]
pub enum EdgeDirection {
    #[default]
    None,
    FirstToSecond,
    SecondToFirst,
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[require(Temperature, Reactivity)]
#[reflect(Component)]
pub struct ReactorEdge {
    pub nodes: (Position, Position),
    pub direction: EdgeDirection,
}

impl ReactorEdge {
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            nodes: (from, to),
            direction: EdgeDirection::None,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct ReactorCore {
    pub cells: HashMap<Position, Entity>,
    pub edges: HashMap<(Position, Position), Entity>,
    pub rows: usize,
    pub columns: usize,
}

impl ReactorCore {
    pub fn iter_all_positions(&self) -> impl Iterator<Item = Position> + '_ {
        let column_radius = (self.columns / 2) as i32;
        let column_rem = (self.columns % 2) as i32;
        let row_radius = (self.rows / 2) as i32;
        let row_rem = (self.rows % 2) as i32;

        (-column_radius..column_radius + column_rem).flat_map(move |x| {
            (-row_radius..row_radius + row_rem).map(move |y| Position::new(x, y))
        })
    }

    pub fn iter_all_positions_with_cells(
        &self,
    ) -> impl Iterator<Item = (Position, Option<Entity>)> + '_ {
        self.iter_all_positions()
            .map(|pos| (pos, self.cells.get(&pos).map(|&entity| entity)))
    }

    pub fn iter_valid_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.iter_all_positions().filter(|pos| pos.is_valid())
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (Position, Entity)> + '_ {
        self.iter_valid_positions()
            .filter_map(move |pos| self.cells.get(&pos).map(|&entity| (pos, entity)))
    }

    pub fn find_edge(&self, from: Position, to: Position) -> Option<Entity> {
        self.edges
            .get(&(from, to))
            .copied()
            .or_else(|| self.edges.get(&(to, from)).copied())
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct ReactorCoreReady;

fn on_add_reactor_core(
    trigger: Trigger<OnAdd, ReactorCore>,
    mut commands: Commands,
    mut cores_q: Query<&mut ReactorCore>,
) -> Result {
    let mut core = cores_q.get_mut(trigger.target())?;

    let mut cells = Vec::new();
    for pos in core.iter_valid_positions() {
        let entity = commands
            .spawn((
                Name::new(format!("Cell {}/{}", pos.x, pos.y)),
                ChildOf(trigger.target()),
                ReactorCell(pos),
            ))
            .observe(events::handle_move_control_rod)
            .id();
        cells.push((pos, entity));
    }

    for (pos, entity) in cells {
        core.cells.insert(pos, entity);
        for other in pos.neighbours() {
            if core.cells.get(&other).is_some() && core.find_edge(pos, other).is_none() {
                let edge_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Edge {}/{} -> {}/{}",
                            pos.x, pos.y, other.x, other.y
                        )),
                        ChildOf(trigger.target()),
                        ReactorEdge::new(pos, other),
                    ))
                    .id();
                core.edges.insert((pos, other), edge_entity);
            }
        }
    }

    commands.entity(trigger.target()).insert(ReactorCoreReady);

    Ok(())
}
