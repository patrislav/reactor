use bevy::prelude::*;

use super::{schedule::RunSimulation, types::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunSimulation,
        (
            update_local_reactivity,
            update_edge_reactivity,
            update_total_reactivity,
            update_temperature,
        )
            .chain(),
    );
    app.add_systems(
        RunSimulation,
        update_edge_temperatures.before(update_temperature),
    );
}

fn update_edge_reactivity(
    cores: Query<&ReactorCore>,
    mut edges: Query<(&mut Reactivity, &ReactorEdge, &ChildOf), Without<ReactorCell>>,
    cells: Query<&LocalReactivity, With<ReactorCell>>,
) -> Result {
    for (mut edge_reactivity, edge, child_of) in &mut edges {
        let core = cores.get(child_of.0)?;
        let Some(first) = core.cells.get(&edge.nodes.0) else {
            warn!("First node not found");
            continue;
        };
        let Some(second) = core.cells.get(&edge.nodes.1) else {
            warn!("Second node not found");
            continue;
        };
        let reactivities = [cells.get(*first)?.0, cells.get(*second)?.0];
        edge_reactivity.0 = (reactivities[0] + reactivities[1]) / 2.;
    }
    Ok(())
}

fn update_local_reactivity(
    config: Res<SimulationConfig>,
    mut query: Query<(&mut LocalReactivity, &ControlRod, &CoolantLevel), With<ReactorCell>>,
) {
    for (mut local_reactivity, control_rod, coolant_level) in &mut query {
        let rod_factor = 1.0 - control_rod.0; // control rods absorb
        let coolant_factor = 1.0 + config.void_reactivity_boost * (1.0 - coolant_level.0); // steam = more reactivity
        local_reactivity.0 = config.base_reactivity * rod_factor * coolant_factor;
    }
}

fn update_total_reactivity(
    config: Res<SimulationConfig>,
    cores: Query<&ReactorCore>,
    mut query: Query<
        (&mut Reactivity, &ReactorCell, &LocalReactivity, &ChildOf),
        Without<ReactorEdge>,
    >,
    edge_reactivities: Query<&Reactivity, (With<ReactorEdge>, Without<ReactorCell>)>,
) -> Result {
    for (mut reactivity, cell, local_reactivity, child_of) in &mut query {
        let core = cores.get(child_of.0)?;

        let mut neighbor_sum = 0.0;
        for pos in cell.0.neighbours() {
            neighbor_sum += match core.find_edge(cell.0, pos) {
                Some(edge) => edge_reactivities.get(edge)?.0,
                None => 0.0,
            }
        }

        let neighbor_boost = neighbor_sum * config.reactivity_neighbor_coupling_factor;
        reactivity.0 = local_reactivity.0 + neighbor_boost;
    }

    Ok(())
}

fn update_edge_temperatures(
    cores: Query<&ReactorCore>,
    mut edges: Query<(&mut Temperature, &ReactorEdge, &ChildOf), Without<ReactorCell>>,
    cells: Query<&Temperature, With<ReactorCell>>,
) -> Result {
    for (mut edge_temperature, edge, child_of) in &mut edges {
        let core = cores.get(child_of.0)?;
        let Some(first) = core.cells.get(&edge.nodes.0) else {
            warn!("First node not found");
            continue;
        };
        let Some(second) = core.cells.get(&edge.nodes.1) else {
            warn!("Second node not found");
            continue;
        };

        // Edge temperature becomes the average of connected cells
        let temps = [cells.get(*first)?.0, cells.get(*second)?.0];
        edge_temperature.0 = (temps[0] + temps[1]) / 2.;
    }
    Ok(())
}

fn update_temperature(
    config: Res<SimulationConfig>,
    cores: Query<&ReactorCore>,
    mut query: Query<
        (
            &mut Temperature,
            &ReactorCell,
            &ChildOf,
            &Reactivity,
            &CoolantFlow,
        ),
        Without<ReactorEdge>,
    >,
    edge_temperatures: Query<&Temperature, (With<ReactorEdge>, Without<ReactorCell>)>,
) -> Result {
    for (mut temperature, cell, child_of, reactivity, coolant_flow) in &mut query {
        let core = cores.get(child_of.0)?;

        let mut neighbor_temp_sum = 0.0;
        for pos in cell.0.neighbours() {
            neighbor_temp_sum += match core.find_edge(cell.0, pos) {
                Some(edge) => edge_temperatures.get(edge)?.0,
                None => 25.0,
            }
        }

        let ambient_temperature = neighbor_temp_sum / 4.;
        let heat_gain = reactivity.0 * config.heat_generation_factor;
        let coolant_temp_diff = temperature.0 - config.coolant_temperature;
        let heat_loss = coolant_flow.0 * config.coolant_efficiency * coolant_temp_diff;
        let ambient_temp_diff = temperature.0 - ambient_temperature;
        let passive_heat_loss = ambient_temp_diff * config.temperature_passive_decay_rate;

        temperature.0 += heat_gain - heat_loss - passive_heat_loss;
    }

    Ok(())
}
