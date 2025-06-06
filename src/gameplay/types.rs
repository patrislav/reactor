use bevy::{platform::collections::HashMap, prelude::*, time::Stopwatch};

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
    app.register_type::<CurrentControlRod>();
    app.register_type::<TargetControlRod>();
}

#[derive(Component, Clone, Copy, Debug, Reflect, Eq, PartialEq)]
#[reflect(Component)]
pub enum Particle {
    Water,
    Steam,
}

impl Particle {
    pub fn color(&self) -> Color {
        match self {
            Particle::Water => Color::from(WATER_COLOR),
            Particle::Steam => Color::from(STEAM_COLOR),
        }
    }
}

#[derive(Component, Clone, Reflect, Default)]
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
pub struct ParticleCount(pub usize);

#[derive(Component, Clone, Copy, Reflect)]
#[require(ParticleCount, CurrentScale)]
#[reflect(Component)]
pub struct Cell(pub Position);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[require(Reactivity)]
#[reflect(Component)]
pub struct CurrentControlRod(pub f32);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct TargetControlRod(pub f32);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct CellIndex(pub usize);

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Reactivity(pub f32);

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

#[derive(Component)]
pub struct Cleanup;

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
