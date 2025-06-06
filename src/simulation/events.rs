use std::marker::PhantomData;

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
    info!("handle_move_control_rod {}", trigger.target());
    let mut control_rod = control_rods.get_mut(trigger.target())?;
    control_rod.0 += trigger.event().0;
    control_rod.0 = control_rod.0.clamp(0.0, 1.0);
    Ok(())
}

#[derive(Event, Clone, Copy, Reflect)]
pub struct PropertyChanged<T, V> {
    pub index: usize,
    pub value: V,
    _phantom: PhantomData<T>,
}

impl<T, V> PropertyChanged<T, V> {
    pub fn new(index: usize, value: V) -> Self {
        Self {
            index,
            value,
            _phantom: PhantomData::<T>,
        }
    }
}

#[derive(Event, Clone, Copy, Reflect)]
pub struct RequestProperty<T, V> {
    pub index: usize,
    _t: PhantomData<T>,
    _v: PhantomData<V>,
}

impl<T, V> RequestProperty<T, V> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            _t: PhantomData::<T>,
            _v: PhantomData::<V>,
        }
    }
}
