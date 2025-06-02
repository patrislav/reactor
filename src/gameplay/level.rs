//! Spawn the main level.

use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

use super::player::{Player, PlayerCamera};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    scene: Handle<Scene>,

    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            scene: assets.load(GltfAssetLabel::Scene(0).from_asset("level.glb")),
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            (
                Name::new("Level Scene"),
                SceneRoot(level_assets.scene.clone()),
            ),
            (
                Name::new("Player"),
                Transform::from_xyz(0.0, 1.5, 0.0),
                Visibility::default(),
                Player,
                children![(
                    Name::new("Camera"),
                    PlayerCamera,
                    Transform::from_xyz(0.0, 1.0, 0.0).looking_to(-Dir3::Z, Dir3::Y),
                    Camera3d::default(),
                    Camera {
                        hdr: true,
                        ..default()
                    },
                    AmbientLight {
                        brightness: 1000.,
                        ..default()
                    }
                )]
            ),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
        ],
    ));
}
