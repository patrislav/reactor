use bevy::prelude::*;
use bevy_inspector_egui::egui::next_tooltip_id;

use crate::screens::game_over::{GameOver, GameOverCause};

use super::{ui::ParticleContainerColor, *};

pub fn plugin(app: &mut App) {
    app.add_systems(
        RunSimulation,
        (turn_steam_into_power, handle_lack_of_power).in_set(PhaseSystems::PowerGeneration),
    );
    app.add_systems(
        Update,
        (
            tick_timers,
            increase_power_demand,
            increase_power_demand_increase_rate,
            handle_lack_of_power_timer,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

#[derive(Component, Clone, Default, Reflect)]
pub struct TicksWithoutPower(pub usize);

fn tick_timers(time: Res<Time>, mut query: Single<&mut NextPowerDemand>) {
    query.tutorial_timer.tick(time.delta());
    if query.tutorial_timer.finished() {
        query.demand_timer.tick(time.delta());
        query.demand_rate_timer.tick(time.delta());
    }
}

fn increase_power_demand(query: Single<(&NextPowerDemand, &mut PowerDemand)>) {
    let (next, mut current) = query.into_inner();

    if next.demand_timer.just_finished() {
        current.0 += next.delta;
    }
}

fn increase_power_demand_increase_rate(mut next: Single<&mut NextPowerDemand>) {
    if next.demand_rate_timer.just_finished() {
        next.delta += 1;
    }
}

fn turn_steam_into_power(
    mut commands: Commands,
    energy_container: Single<
        (
            Entity,
            &ParticleContainer,
            &GlobalTransform,
            &PowerDemand,
            &NextPowerDemand,
            &mut TicksWithoutPower,
        ),
        (With<EnergyContainer>, Without<SteamContainer>),
    >,
    steam_container: Single<
        (&mut ParticleContainer, &GlobalTransform),
        (With<SteamContainer>, Without<EnergyContainer>),
    >,
) {
    let (
        energy_entity,
        energy_container,
        energy_transform,
        demand,
        next_demand,
        mut ticks_without_power,
    ) = energy_container.into_inner();
    let (mut steam_container, steam_transform) = steam_container.into_inner();

    let diff = if energy_container.count < demand.0 {
        demand.0 - energy_container.count
    } else {
        0
    };
    let count = steam_container.count.min(diff + next_demand.delta);

    for _ in 0..count {
        steam_container.count -= 1;

        let particle_transform = energy_transform.affine().inverse() * steam_transform.affine();
        let mut particle_transform = Transform::from_matrix(particle_transform.into());
        particle_transform.translation.z = 20.0;

        commands.spawn((
            Name::new("Energy particle"),
            Particle::Energy,
            particle_transform,
            ChildOf(energy_entity),
            EasedMotion {
                from: particle_transform,
                to: Transform::from_xyz(0.0, 0.0, 20.0),
                timer: Timer::from_seconds(1.0, TimerMode::Once),
                easing: EaseFunction::CubicInOut,
            },
        ));
    }

    if diff > count {
        ticks_without_power.0 += 1;
    } else {
        ticks_without_power.0 = 0;
    }
}

fn handle_lack_of_power(
    mut commands: Commands,
    query: Single<(Entity, &TicksWithoutPower)>,
    energy_container: Single<&ParticleContainer, With<EnergyContainer>>,
) {
    let (entity, ticks) = query.into_inner();
    if ticks.0 > 20 {
        commands.trigger(GameOver {
            cause: GameOverCause::NotEnoughPower,
            power_generated: energy_container.count,
        });
    } else if ticks.0 > 5 {
        commands
            .entity(entity)
            .try_insert(LackOfPowerTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    } else {
        commands
            .entity(entity)
            .remove::<LackOfPowerTimer>()
            .try_insert(ParticleContainerColor(URANIUM_COLOR));
    }
}

#[derive(Component)]
struct LackOfPowerTimer(Timer);

fn handle_lack_of_power_timer(
    mut commands: Commands,
    time: Res<Time>,
    query: Single<(Entity, &mut LackOfPowerTimer)>,
) {
    let (entity, mut timer) = query.into_inner();

    timer.0.tick(time.delta());

    let t = timer.0.fraction();
    let blend = if t <= 0.5 {
        t / 0.5 // 0.0 -> 1.0
    } else {
        1.0 - ((t - 0.5) / 0.5) // 1.0 -> 0.0
    };
    commands.entity(entity).try_insert(ParticleContainerColor(
        Color::from(WARNING_COLOR).mix(&URANIUM_COLOR, blend),
    ));
}
