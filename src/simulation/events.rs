use super::types::*;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<MoveControlRod>();
}

#[derive(Event)]
pub struct MoveControlRod(pub f32);

pub(super) fn handle_move_control_rod(
    trigger: Trigger<MoveControlRod>,
    mut control_rods: Query<&mut ControlRod, With<ReactorCell>>,
) -> Result {
    let mut control_rod = control_rods.get_mut(trigger.target())?;
    control_rod.0 += trigger.event().0;
    control_rod.0 = control_rod.0.clamp(0.0, 1.0);
    Ok(())
}
