use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

use super::{interaction::InteractionCandidate, player::Player};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CrosshairAssets>();
    app.load_resource::<CrosshairAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_crosshair);
    app.add_systems(Update, update_crosshair.run_if(in_state(Screen::Gameplay)));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct CrosshairAssets {
    #[dependency]
    dot: Handle<Image>,

    #[dependency]
    square: Handle<Image>,
}

impl FromWorld for CrosshairAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            dot: assets.load("ui/crosshair_dot.png"),
            square: assets.load("ui/crosshair_square.png"),
        }
    }
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
struct Crosshair;

fn spawn_crosshair(mut commands: Commands, assets: Res<CrosshairAssets>) {
    commands
        .spawn((
            Name::new("Crosshair"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            StateScoped(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Crosshair Image"),
                Crosshair,
                ImageNode::new(assets.dot.clone()),
            ));
        });
}

fn update_crosshair(
    assets: Res<CrosshairAssets>,
    candidate: Single<&InteractionCandidate, With<Player>>,
    mut crosshair: Single<&mut ImageNode, With<Crosshair>>,
) {
    crosshair.image = match candidate.0 {
        None => assets.dot.clone(),
        Some(_) => assets.square.clone(),
    }
}
