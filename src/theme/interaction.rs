use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, audio::sound_effect};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InteractionPalette>();
    app.add_systems(Update, apply_interaction_palette);

    app.register_type::<InteractionAssets>();
    app.load_resource::<InteractionAssets>();
    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
    app.add_observer(set_font);
}

#[derive(Component, Copy, Clone, Reflect, Default)]
pub struct PlaysHoverSound;

#[derive(Component, Copy, Clone, Reflect, Default)]
pub struct PlaysClickSound;

#[derive(Component, Copy, Clone, Reflect, Default)]
pub struct UseBoldFont;

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut background) in &mut palette_query {
        *background = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct InteractionAssets {
    #[dependency]
    hover: Handle<AudioSource>,
    #[dependency]
    click: Handle<AudioSource>,
    #[dependency]
    font_light: Handle<Font>,
    #[dependency]
    font_bold: Handle<Font>,
}

impl FromWorld for InteractionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hover: assets.load("audio/sound_effects/button_hover.ogg"),
            click: assets.load("audio/sound_effects/button_click.ogg"),
            font_light: assets.load("fonts/Lato-Light.ttf"),
            font_bold: assets.load("fonts/Lato-Bold.ttf"),
        }
    }
}

fn play_on_hover_sound_effect(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), Or<(With<Interaction>, With<PlaysHoverSound>)>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(interaction_assets.hover.clone()));
    }
}

fn play_on_click_sound_effect(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), Or<(With<Interaction>, With<PlaysClickSound>)>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(interaction_assets.click.clone()));
    }
}

fn set_font(
    trigger: Trigger<OnAdd, TextFont>,
    assets: Res<InteractionAssets>,
    mut query: Query<(&mut TextFont, Option<&UseBoldFont>)>,
) {
    if let Ok((mut text_font, use_bold_font)) = query.get_mut(trigger.target()) {
        text_font.font = if use_bold_font.is_some() {
            assets.font_bold.clone()
        } else {
            assets.font_light.clone()
        }
    }
}
