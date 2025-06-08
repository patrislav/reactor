use bevy::{color::palettes::css, prelude::*};

pub const CELL_ROWS: usize = 7;
pub const CELL_COLUMNS: usize = 7;
pub const CELL_RADIUS: f32 = 50.;
pub const CELL_OUTER_SIZE: f32 = 95.;

pub const PARTICLE_RADIUS: f32 = 5.;
pub const NEUTRON_RADIUS: f32 = 2.5;
pub const CONTROL_ROD_RADIUS: f32 = 30.0;
pub const FUEL_ROD_RADIUS: f32 = 25.0;

pub const NEUTRON_LIFETIME_SEC: f32 = 45.0;
pub const INCREASE_POWER_DEMAND_SEC: f32 = 1.0;
pub const INCREASE_POWER_DEMAND_INCREASE_RATE_SEC: f32 = 10.0;
pub const TUTORIAL_SEC: f32 = 10.0;
pub const CONTAINER_SPACING: f32 = 5.;
pub const COLLISION_LEEWAY: f32 = 3.;
pub const INITIAL_URANIUM_TO_XENON_RATIO: f32 = 0.65;

pub const CONTROL_ROD_INSERTION_SPEED: f32 = 0.1; // per sec
pub const PARTICLE_ANGULAR_SPEED: f32 = 3.; // per sec
pub const PARTICLE_DISTANCE_SPEED: f32 = 3.; // per sec
pub const NEUTRON_SPEED: f32 = 65.; // per sec
pub const SCALE_SPEED: f32 = 10.; // per sec

pub const MAX_NEUTRONS_RELEASED_PER_TICK: usize = 3;
pub const NEUTRON_SPAWN_CHANCE: f32 = 0.6;
pub const WATER_CREATED_PER_TICK: usize = 200;
pub const MAX_WATER_STORED: usize = 2000;
pub const STEAM_GENERATED_PER_WATER: usize = 2;
pub const STEAM_VENTED_PER_TICK: usize = 3;
pub const XENON_SPAWN_CHANCE_PER_TICK: f32 = 0.1;

pub const PRESSURE_WARN_LEVEL: usize = 30;
pub const PRESSURE_EXPLOSION_LEVEL: usize = 50;

pub const CELL_COLOR: Srgba = css::GRAY;
pub const WATER_COLOR: Srgba = css::LIGHT_SKY_BLUE;
pub const STEAM_COLOR: Srgba = css::WHITE;
pub const NEUTRON_COLOR: Srgba = css::MAGENTA;
pub const URANIUM_COLOR: Color = Color::srgb(0.85, 0.85, 0.65);
pub const XENON_COLOR: Color = Color::srgb(0.60, 0.88, 0.81);
pub const CONTROL_ROD_COLOR_INSERTED: Color = Color::srgb(0.85, 0.67, 0.67);
pub const CONTROL_ROD_COLOR: Color = Color::srgb(0.23, 0.175, 0.175);
pub const WARNING_COLOR: Srgba = css::RED;
