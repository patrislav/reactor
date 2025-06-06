use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, simulation::types::*};

pub mod edges;
pub mod material;
pub mod view;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ReactorViewAssets>();
    app.load_resource::<ReactorViewAssets>();

    app.add_plugins(material::plugin);

    app.add_plugins(view::plugin::<1>);

    app.add_systems(Update, (update_displayed_reactivity,).chain());
    app.add_systems(Update, (update_displayed_temperature,).chain());
    app.add_systems(Update, (update_displayed_control_rod,).chain());
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ReactorViewAssets {
    #[dependency]
    pub arrow_texture: Handle<Image>,
}

impl FromWorld for ReactorViewAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            arrow_texture: assets.load("textures/arrow.png"),
        }
    }
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
