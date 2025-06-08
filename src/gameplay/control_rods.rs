use bevy::prelude::*;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_observer(setup_control_rod_on_add)
        .add_observer(hide_movement_indicators);
    app.add_systems(
        Update,
        (
            update_insertions,
            update_insertion_indicators,
            update_movement_indicators,
            update_materials,
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn setup_control_rod_on_add(trigger: Trigger<OnAdd, ControlRod>, mut commands: Commands) {
    commands.entity(trigger.target()).observe(on_click);
}

fn on_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut query: Query<(Option<&mut ControlRodMovement>, &ControlRodInsertion)>,
) {
    if let Ok((maybe_movement, insertion)) = query.get_mut(trigger.target()) {
        if let Some(mut movement) = maybe_movement {
            movement.reverse();
        } else if insertion.0 > 0.5 {
            commands
                .entity(trigger.target())
                .insert(ControlRodMovement::Down);
        } else {
            commands
                .entity(trigger.target())
                .insert(ControlRodMovement::Up);
        }
    }
}

fn update_insertions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ControlRodInsertion, &ControlRodMovement)>,
) {
    for (entity, mut insertion, movement) in &mut query {
        insertion.0 += time.delta_secs()
            * match movement {
                ControlRodMovement::Up => CONTROL_ROD_INSERTION_SPEED,
                ControlRodMovement::Down => -CONTROL_ROD_INSERTION_SPEED,
            };

        if insertion.0 >= 1.0 || insertion.0 <= 0.0 {
            insertion.0 = insertion.0.clamp(0.0, 1.0);
            commands.entity(entity).remove::<ControlRodMovement>();
        }
    }
}

fn update_movement_indicators(
    mut commands: Commands,
    control_rods: Query<(&ControlRodMovement, &Children), Changed<ControlRodMovement>>,
    indicators: Query<(), With<ControlRodMovementIndicator>>,
) {
    for (movement, children) in &control_rods {
        for &entity in children.into_iter() {
            if indicators.contains(entity) {
                let angle = match movement {
                    ControlRodMovement::Up => 0.0,
                    ControlRodMovement::Down => PI,
                };
                commands.entity(entity).insert((
                    Transform::from_rotation(Quat::from_rotation_z(angle)),
                    Visibility::Visible,
                ));
            }
        }
    }
}

fn hide_movement_indicators(
    trigger: Trigger<OnRemove, ControlRodMovement>,
    control_rods: Query<&Children, With<ControlRod>>,
    mut commands: Commands,
    indicators: Query<(), With<ControlRodMovementIndicator>>,
) {
    if let Ok(children) = control_rods.get(trigger.target()) {
        for &entity in children.into_iter() {
            if indicators.contains(entity) {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
    }
}

fn update_insertion_indicators(
    control_rods: Query<
        (&ControlRodInsertion, &Children),
        (With<ControlRod>, Changed<ControlRodInsertion>),
    >,
    mut indicators: Query<&mut Text2d, With<ControlRodInsertionIndicator>>,
) {
    for (insertion, children) in &control_rods {
        for &entity in children.into_iter() {
            if let Ok(mut text) = indicators.get_mut(entity) {
                text.0 = format!("{:.0}%", insertion.0 * 100.);
            }
        }
    }
}

fn update_materials(
    query: Query<
        (&ControlRodInsertion, &MeshMaterial2d<ColorMaterial>),
        (With<ControlRod>, Changed<ControlRodInsertion>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (insertion, material) in &query {
        if let Some(material) = materials.get_mut(material) {
            material.color = CONTROL_ROD_COLOR.mix(&CONTROL_ROD_COLOR_INSERTED, insertion.0);
        }
    }
}
