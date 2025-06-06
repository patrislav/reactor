//! Spawn the main level.

use avian3d::prelude::{Collider, ColliderConstructor};
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
};

use crate::{asset_tracking::LoadResource, audio::music, screens::Screen};

use super::player::{Player, PlayerCamera};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.add_systems(OnEnter(Screen::GameplayLoading), spawn_level);
    app.add_systems(OnEnter(Screen::Gameplay), spawn_player);
    app.add_systems(
        Update,
        move_to_gameplay_once_loaded.run_if(in_state(Screen::GameplayLoading)),
    );
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

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
struct Level;

/// A system that spawns the main level.
fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    commands.spawn((
        Name::new("Level"),
        Level,
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            (
                Name::new("Level Scene"),
                SceneRoot(level_assets.scene.clone()),
            ),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
        ],
    ));
}

fn spawn_player(mut commands: Commands, level: Single<Entity, With<Level>>) {
    commands.spawn((
        Name::new("Player"),
        ChildOf(level.into_inner()),
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
            },
            Tonemapping::TonyMcMapface,
            Bloom::NATURAL,
        )],
    ));
}

fn move_to_gameplay_once_loaded(
    mut screen: ResMut<NextState<Screen>>,
    collider_constructors: Query<(), With<ColliderConstructor>>,
) {
    if collider_constructors.iter().len() == 0 {
        screen.set(Screen::Gameplay);
    }
}
