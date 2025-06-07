use std::time::Duration;

use bevy::{
    ecs::schedule::{ExecutorKind, ScheduleLabel},
    prelude::*,
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunSimulation;

pub fn plugin(app: &mut App) {
    let mut schedule = Schedule::new(RunSimulation);
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);

    app.add_schedule(schedule);
    app.insert_resource(Time::new_with(Simulation::default()));
    app.add_systems(Main, run_simulation_schedule);
}

#[derive(Debug, Copy, Clone)]
pub struct Simulation {
    timestep: Duration,
    overstep: Duration,
}

impl Simulation {
    /// Corresponds to 1 Hz.
    const DEFAULT_TIMESTEP: Duration = Duration::from_millis(500);

    fn accumulate(&mut self, delta: Duration) {
        self.overstep += delta;
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            timestep: Simulation::DEFAULT_TIMESTEP,
            overstep: Duration::ZERO,
        }
    }
}

fn run_simulation_schedule(world: &mut World) {
    let delta = world.resource::<Time<Virtual>>().delta();
    world
        .resource_mut::<Time<Simulation>>()
        .context_mut()
        .accumulate(delta);

    // Run the schedule until we run out of accumulated time
    let _ = world.try_schedule_scope(RunSimulation, |world, schedule| {
        while expend_simulation(world.resource_mut::<Time<Simulation>>().into_inner()) {
            *world.resource_mut::<Time>() = world.resource::<Time<Simulation>>().as_generic();
            schedule.run(world);
        }
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}

fn expend_simulation(time: &mut Time<Simulation>) -> bool {
    let timestep = time.context().timestep;
    if let Some(new_value) = time.context_mut().overstep.checked_sub(timestep) {
        // reduce accumulated and increase elapsed by period
        time.context_mut().overstep = new_value;
        time.advance_by(timestep);
        true
    } else {
        // no more periods left in accumulated
        false
    }
}
