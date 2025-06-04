use bevy::{prelude::*, text::cosmic_text::Change};

pub fn plugin(app: &mut App) {
    app.register_type::<ChangeLedMaterial>();
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
struct ChangeLedMaterial {
    condition: Condition,
}

#[derive(Clone, Copy, Default, Reflect)]
enum Condition {
    #[default]
    None,
    ControlRodSelected(usize),
}
