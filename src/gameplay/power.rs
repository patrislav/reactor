use bevy::prelude::*;

use crate::screens::game_over::{GameOver, GameOverCause};

use super::{ui::ParticleContainerColor, *};

pub fn plugin(app: &mut App) {
    app.add_systems(
        RunSimulation,
        (turn_steam_into_power, handle_lack_of_power).in_set(PhaseSystems::PowerGeneration),
    );
    app.add_systems(
        Update,
        (increase_power_demand, handle_lack_of_power_timer).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Clone, Default, Reflect)]
pub struct TicksWithoutPower(pub usize);

fn increase_power_demand(time: Res<Time>, query: Single<(&mut NextPowerDemand, &mut PowerDemand)>) {
    let (mut next, mut current) = query.into_inner();

    next.timer.tick(time.delta());

    if next.timer.just_finished() {
        current.0 = next.demand;
        next.demand *= 2;
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
            &mut TicksWithoutPower,
        ),
        (With<EnergyContainer>, Without<SteamContainer>),
    >,
    steam_container: Single<
        (&mut ParticleContainer, &GlobalTransform),
        (With<SteamContainer>, Without<EnergyContainer>),
    >,
) {
    let (energy_entity, energy_container, energy_transform, demand, mut ticks_without_power) =
        energy_container.into_inner();
    let (mut steam_container, steam_transform) = steam_container.into_inner();

    let diff = demand.0 - energy_container.count;
    let count = steam_container.count.min(diff);

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
            .insert(LackOfPowerTimer(Timer::from_seconds(1.0, TimerMode::Once)));
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
    let blend = if t <= 0.25 {
        t / 0.25 // 0.0 -> 1.0
    } else {
        1.0 - ((t - 0.25) / 0.75) // 1.0 -> 0.0
    };
    commands.entity(entity).insert(ParticleContainerColor(
        Color::from(WARNING_COLOR).mix(&URANIUM_COLOR, blend),
    ));

    if timer.0.just_finished() {
        commands.entity(entity).remove::<LackOfPowerTimer>();
    }
}
