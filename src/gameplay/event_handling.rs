use bevy::prelude::*;

use crate::simulation::{
    events::{MoveControlRod, PropertyChanged, RequestProperty},
    types::{ReactorCell, ReactorCellIndex, ReactorCore},
};

use super::game_object::ControlRodStatus;

pub fn plugin(app: &mut App) {
    app.add_event::<SelectControlRod>();
    app.add_event::<MoveSelectedControlRod>();

    app.add_observer(on_add_reactor_core);
    app.add_observer(on_select_control_rod);
    app.add_observer(on_move_selected_control_rod);
    app.add_observer(on_request_control_rod_status);
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
struct SelectedControlRod(Option<Entity>);

#[derive(Event, Clone, Copy, Reflect)]
pub struct SelectControlRod(pub usize);

#[derive(Event, Clone, Copy, Reflect)]
pub struct MoveSelectedControlRod(pub f32);

fn on_request_control_rod_status(
    trigger: Trigger<RequestProperty<ReactorCell, ControlRodStatus>>,
    mut commands: Commands,
    core: Single<(&ReactorCore, &SelectedControlRod)>,
) {
    let index = trigger.event().index;
    let (core, selected_cell) = core.into_inner();
    let (_, entity) = core.cells[index];
    commands.trigger(PropertyChanged::<ReactorCell, ControlRodStatus>::new(
        index,
        if selected_cell.0.is_some_and(|cell| cell == entity) {
            ControlRodStatus::Selected
        } else {
            ControlRodStatus::Default
        },
    ));
}

fn on_select_control_rod(
    trigger: Trigger<SelectControlRod>,
    mut commands: Commands,
    mut core: Single<(&ReactorCore, &mut SelectedControlRod)>,
    cells: Query<&ReactorCellIndex>,
) -> Result {
    let index = trigger.event().0;
    let (core, mut selected_cell) = core.into_inner();
    let (_, new_selected_entity) = core.cells[index];

    if let Some(selected_entity) = selected_cell.0 {
        if selected_entity == new_selected_entity {
            return Ok(());
        }

        let current_index = cells.get(selected_entity)?;

        info!("Triggering PriopertyChanged for currently selected cell");
        commands.trigger(PropertyChanged::<ReactorCell, ControlRodStatus>::new(
            current_index.0,
            ControlRodStatus::Default,
        ));
    }

    info!("Triggering PriopertyChanged for new cell");
    selected_cell.0 = Some(new_selected_entity);
    commands.trigger(PropertyChanged::<ReactorCell, ControlRodStatus>::new(
        index,
        ControlRodStatus::Selected,
    ));
    Ok(())
}

fn on_move_selected_control_rod(
    trigger: Trigger<MoveSelectedControlRod>,
    mut commands: Commands,
    selected_cell: Single<&SelectedControlRod, With<ReactorCore>>,
) {
    if let Some(entity) = selected_cell.0 {
        commands.trigger_targets(MoveControlRod(trigger.event().0), entity);
    }
}

fn on_add_reactor_core(trigger: Trigger<OnAdd, ReactorCore>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(SelectedControlRod::default());
}
