use bevy::{platform::collections::HashMap, prelude::*};

use super::events;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Temperature>()
        .register_type::<CoolantLevel>()
        .register_type::<CoolantFlow>()
        .register_type::<Reactivity>()
        .register_type::<LocalReactivity>()
        .register_type::<ControlRod>()
        .register_type::<Pressure>()
        .register_type::<SteamOutput>()
        .register_type::<CoolantValve>()
        .register_type::<CoolantCircuit>()
        .register_type::<ReactorCell>()
        .register_type::<ReactorEdge>()
        .register_type::<ReactorCoolantEdge>()
        .init_resource::<SimulationConfig>()
        .add_observer(on_add_reactor_core);
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct SimulationConfig {
    /// Base reactivity per cell with no control rod inserted
    pub base_reactivity: f32,

    /// Additional reactivity when coolant is low
    pub void_reactivity_boost: f32,

    /// How much neighboring cell reactivity affects local reactivity
    pub reactivity_neighbor_coupling_factor: f32,

    /// How much heat is generated per unit of total reactivity
    pub heat_generation_factor: f32,

    /// Temperature of incoming coolant
    pub coolant_temperature: f32,

    /// How much heat is removed per tick per unit of coolant
    pub coolant_efficiency: f32,

    /// Passive temperature decay rate per tick toward ambient temperature
    pub temperature_passive_decay_rate: f32,

    /// Energy generated per degree above boiling point
    pub energy_per_heat_unit: f32,

    /// Energy required to boil 1.0 of coolant into 1.0 of steam
    pub energy_required_per_unit: f32,

    /// How much volume 1.0 unit of steam takes up relative to coolant
    pub steam_expansion_ratio: f32,

    /// Exponent for pressure curve
    pub pressure_curve_exponent: f32,

    /// Atmospheric pressure used as baseline in atmospheres
    pub nominal_pressure: f32,

    /// Scales raw pressure result into usable simulation units
    pub pressure_scale: f32,

    /// How much steam can be pulled per tick per unit pressure
    pub steam_pull_factor: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            base_reactivity: 1.0,
            void_reactivity_boost: 1.5,
            reactivity_neighbor_coupling_factor: 0.2,
            heat_generation_factor: 5.0,
            coolant_temperature: 40.0,
            coolant_efficiency: 0.1,
            temperature_passive_decay_rate: 0.01,
            energy_per_heat_unit: 10.0,
            energy_required_per_unit: 2000.0,
            steam_expansion_ratio: 100.0,
            pressure_curve_exponent: 1.2,
            nominal_pressure: 1.0,
            pressure_scale: 0.01,
            steam_pull_factor: 0.2,
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

/// How full the cell is with steam, 0.0-1.0.
#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct SteamLevel(pub f32);

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct SteamOutput(pub f32);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Pressure(pub f32);

impl Default for Pressure {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct SteamPullCapacity(pub f32);

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

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct CoolantValve {
    pub open: bool,
    pub cells: Vec<Entity>,
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct CoolantCircuit {
    pub power: f32,
    pub valves: Vec<Entity>,
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
    Pressure,
    CoolantLevel,
    CoolantFlow,
    SteamPullCapacity,
    SteamLevel,
    SteamOutput
)]
pub struct ReactorCell(pub Position);

#[derive(Component, Clone, Copy, Default, Reflect, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub struct ReactorCellIndex(pub usize);

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

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct ReactorCoolantEdge;

#[derive(Component, Clone)]
pub struct ReactorCore {
    pub cells: [(Position, Entity); 24],
    pub cells_by_pos: HashMap<Position, Entity>,
    pub valve_by_cell: HashMap<Entity, Entity>,
    pub edges: HashMap<(Position, Position), Entity>,
    pub rows: usize,
    pub columns: usize,
}

impl Default for ReactorCore {
    fn default() -> Self {
        Self {
            cells: [(Position::new(0, 0), Entity::PLACEHOLDER); 24],
            cells_by_pos: HashMap::new(),
            valve_by_cell: HashMap::new(),
            edges: HashMap::new(),
            rows: 7,
            columns: 7,
        }
    }
}

impl ReactorCore {
    pub fn iter_all_positions(&self) -> impl Iterator<Item = Position> + '_ {
        let column_radius = (self.columns / 2) as i32;
        let column_rem = (self.columns % 2) as i32;
        let row_radius = (self.rows / 2) as i32;
        let row_rem = (self.rows % 2) as i32;

        (-row_radius..row_radius + row_rem).flat_map(move |y| {
            (-column_radius..column_radius + column_rem).map(move |x| Position::new(x, y))
        })
    }

    pub fn iter_all_positions_with_cells(
        &self,
    ) -> impl Iterator<Item = (Position, Option<Entity>)> + '_ {
        self.iter_all_positions()
            .map(|pos| (pos, self.cells_by_pos.get(&pos).copied()))
    }

    pub fn iter_valid_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.iter_all_positions().filter(|pos| pos.is_valid())
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (Position, Entity)> + '_ {
        self.iter_valid_positions()
            .filter_map(move |pos| self.cells_by_pos.get(&pos).map(|&entity| (pos, entity)))
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
    for (index, pos) in core.iter_valid_positions().enumerate() {
        let entity = commands
            .spawn((
                Name::new(format!("Cell {}/{}", pos.x, pos.y)),
                ChildOf(trigger.target()),
                ReactorCell(pos),
                ReactorCellIndex(index),
            ))
            .observe(events::handle_move_control_rod)
            .id();
        cells.push((pos, entity));
    }

    let circuits = [
        [1usize, 5, 9, 2, 6, 3, 4, 8, 12, 11, 15, 18],
        [14, 10, 7, 21, 17, 13, 24, 20, 16, 23, 19, 22],
    ];

    for circuit in circuits {
        circuit
            .chunks(3)
            .enumerate()
            .for_each(|(valve_index, valve_cell_indices)| {
                let cells: Vec<Entity> = valve_cell_indices
                    .iter()
                    .map(|index| cells[index - 1].1)
                    .collect();
                let valve_entity = commands
                    .spawn((
                        Name::new(format!("Valve {}", valve_index)),
                        ChildOf(trigger.target()),
                        CoolantValve {
                            open: false,
                            cells: cells.clone(),
                        },
                    ))
                    .id();
                for cell_entity in cells {
                    core.valve_by_cell.insert(cell_entity, valve_entity);
                }
            });
    }

    for (index, (pos, entity)) in cells.iter().copied().enumerate() {
        core.cells[index] = (pos, entity);
        core.cells_by_pos.insert(pos, entity);

        for other in pos.neighbours() {
            if core.cells_by_pos.get(&other).is_some() && core.find_edge(pos, other).is_none() {
                let other_entity = core.cells_by_pos.get(&other);
                let this_valve = core.valve_by_cell.get(&entity).copied();
                let other_valve = other_entity
                    .and_then(|entity| core.valve_by_cell.get(entity))
                    .copied();

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

                if this_valve == other_valve {
                    commands.entity(edge_entity).insert(ReactorCoolantEdge);
                }
            }
        }
    }

    commands.entity(trigger.target()).insert(ReactorCoreReady);

    Ok(())
}
