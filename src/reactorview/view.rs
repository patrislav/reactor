use bevy::prelude::*;

pub fn plugin<const N: usize>(app: &mut App) {}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
struct ReactorViewLink<const N: usize>(Entity);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub enum DisplayMode {
    #[default]
    Loading,
    Control,
    Cooling,
    Temperature,
    Pressure,
}
