use std::time::Duration;

use avian2d::prelude::PhysicsLayer;
use bevy::{platform::collections::HashMap, prelude::*, time::Stopwatch};
use rand::Rng;

use super::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Neutron>();
    app.register_type::<Particle>();
    app.register_type::<Expiry>();
    app.register_type::<Lifetime>();
    app.register_type::<CurrentAngle>();
    app.register_type::<TargetAngle>();
    app.register_type::<CurrentScale>();
    app.register_type::<TargetScale>();
}

#[derive(Component, Clone, Copy, Debug, Reflect, Eq, PartialEq)]
#[reflect(Component)]
pub enum Particle {
    Water(bool),
    Steam,
    Energy,
}

impl Particle {
    pub fn color(&self) -> Color {
        match self {
            Particle::Water(_) => Color::from(WATER_COLOR),
            Particle::Steam => Color::from(STEAM_COLOR),
            Particle::Energy => URANIUM_COLOR,
        }
    }
}

#[derive(Component, Clone, Reflect, Default, Eq, PartialEq)]
#[reflect(Component)]
#[require(CurrentAngle, CurrentDistance)]
pub enum Neutron {
    #[default]
    Active,
    Dying,
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct Expiry(pub Timer);

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct Lifetime(pub Stopwatch);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct CurrentScale(pub f32);

impl Default for CurrentScale {
    fn default() -> Self {
        CurrentScale(1.0)
    }
}

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct TargetScale(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct CurrentAngle(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct TargetAngle(pub f32);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Origin(pub Entity);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct CurrentDistance(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct TargetDistance(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct ParticleCount(usize);

impl ParticleCount {
    pub fn get(&self) -> usize {
        self.0
    }

    pub fn increment(&mut self, amount: usize) {
        self.0 += amount;
    }

    pub fn decrement(&mut self, amount: usize) {
        if amount > self.0 {
            self.0 = 0;
        } else {
            self.0 -= amount;
        }
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[require(ParticleCount, CurrentScale, CellColor)]
#[reflect(Component)]
pub struct Cell(pub Position);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct CellColor(pub Color);

impl Default for CellColor {
    fn default() -> Self {
        Self(CELL_COLOR.into())
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct InCell;

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ControlRod(pub Position);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ControlRodInsertion(pub f32);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub enum ControlRodMovement {
    Up,
    Down,
}

impl ControlRodMovement {
    pub fn reverse(&mut self) {
        *self = match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component, Clone, Copy, Reflect, Eq, PartialEq)]
#[reflect(Component)]
pub enum FuelRod {
    Uranium,
    Xenon,
}

impl FuelRod {
    pub fn random(uranium_chance: f32) -> Self {
        let mut rng = rand::rng();
        if rng.random_range(0.0..1.0) < uranium_chance {
            Self::Uranium
        } else {
            Self::Xenon
        }
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ControlRodMovementIndicator;

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ControlRodInsertionIndicator;

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct CellIndex(pub usize);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Reactivity(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
pub struct PowerDemand(pub usize);

#[derive(Component, Clone, Reflect)]
pub struct NextPowerDemand {
    pub delta: usize,
    pub demand_timer: Timer,
    pub demand_rate_timer: Timer,
    pub tutorial_timer: Timer,
}

impl Default for NextPowerDemand {
    fn default() -> Self {
        Self {
            delta: 1,
            demand_timer: Timer::new(
                Duration::from_secs_f32(INCREASE_POWER_DEMAND_SEC),
                TimerMode::Repeating,
            ),
            demand_rate_timer: Timer::new(
                Duration::from_secs_f32(INCREASE_POWER_DEMAND_INCREASE_RATE_SEC),
                TimerMode::Repeating,
            ),
            tutorial_timer: Timer::new(Duration::from_secs_f32(TUTORIAL_SEC), TimerMode::Once),
        }
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

#[derive(Event)]
pub struct Cleanup(pub Entity);

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct ReactorCore {
    pub cells: [(Position, Entity); 24],
    pub cells_by_pos: HashMap<Position, Entity>,
    pub edges: HashMap<(Position, Position), Entity>,
    pub rows: usize,
    pub columns: usize,
}

impl Default for ReactorCore {
    fn default() -> Self {
        Self {
            cells: [(Position::new(0, 0), Entity::PLACEHOLDER); 24],
            cells_by_pos: HashMap::new(),
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

    pub fn iter_cell_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.iter_all_positions().filter(|pos| pos.is_valid())
    }

    pub fn iter_control_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.iter_all_positions().filter(|pos| !pos.is_valid())
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (Position, Entity)> + '_ {
        self.iter_cell_positions()
            .filter_map(move |pos| self.cells_by_pos.get(&pos).map(|&entity| (pos, entity)))
    }

    pub fn find_edge(&self, from: Position, to: Position) -> Option<Entity> {
        self.edges
            .get(&(from, to))
            .copied()
            .or_else(|| self.edges.get(&(to, from)).copied())
    }
}

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Particle,
    Neutron,
    ControlRod,
    FuelRod,
}
