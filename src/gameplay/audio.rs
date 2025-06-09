use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::{
    asset_tracking::LoadResource,
    audio::{Music, music, sound_effect_volume},
    screens::Screen,
};

use super::{BoilWaterParticle, NeutronCollision};

pub fn plugin(app: &mut App) {
    app.init_resource::<AudioAssets>();
    app.load_resource::<AudioAssets>();

    app.add_systems(OnEnter(Screen::Title), play_music);

    app.add_observer(on_neutron_collision);
    app.add_observer(on_boil_water);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct AudioAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    hit1: Handle<AudioSource>,
    #[dependency]
    hit2: Handle<AudioSource>,
    #[dependency]
    hit3: Handle<AudioSource>,
    #[dependency]
    hit4: Handle<AudioSource>,
    #[dependency]
    water1: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/game.ogg"),
            hit1: assets.load("audio/sound_effects/hit1.wav"),
            hit2: assets.load("audio/sound_effects/hit2.wav"),
            hit3: assets.load("audio/sound_effects/hit3.wav"),
            hit4: assets.load("audio/sound_effects/hit4.wav"),
            water1: assets.load("audio/sound_effects/water1.wav"),
        }
    }
}

fn on_neutron_collision(
    _: Trigger<NeutronCollision>,
    mut commands: Commands,
    assets: Res<AudioAssets>,
) {
    let mut rng = rand::rng();
    let hits = [
        assets.hit1.clone(),
        assets.hit2.clone(),
        assets.hit3.clone(),
        assets.hit4.clone(),
    ];
    if let Some(handle) = hits.choose(&mut rng) {
        commands.spawn(sound_effect_volume(handle.clone(), 0.4));
    }
}

fn on_boil_water(_: Trigger<BoilWaterParticle>, mut commands: Commands, assets: Res<AudioAssets>) {
    commands.spawn(sound_effect_volume(assets.water1.clone(), 0.7));
}

fn play_music(mut commands: Commands, query: Query<(), With<Music>>, assets: Res<AudioAssets>) {
    if query.is_empty() {
        commands.spawn(music(assets.music.clone()));
    }
}
