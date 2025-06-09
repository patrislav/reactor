use std::f32::consts::TAU;

use avian2d::prelude::{CollisionEventsEnabled, LayerMask};
use bevy::prelude::*;
use rand::Rng;

use super::*;
use crate::{
    PausableSystems,
    screens::game_over::{GameOver, GameOverCause},
};

pub fn plugin(app: &mut App) {
    app.init_state::<Phase>();
    app.configure_sets(
        RunSimulation,
        (
            PhaseSystems::PowerGeneration
                .run_if(in_state(Phase::PowerGeneration))
                .run_if(in_state(Screen::Gameplay)),
            PhaseSystems::WaterFlow
                .run_if(in_state(Phase::WaterFlow))
                .run_if(in_state(Screen::Gameplay)),
            PhaseSystems::NeutronRelease
                .run_if(in_state(Phase::NeutronRelease))
                .run_if(in_state(Screen::Gameplay)),
            PhaseSystems::SteamVenting
                .run_if(in_state(Phase::SteamVenting))
                .run_if(in_state(Screen::Gameplay)),
        )
            .in_set(PausableSystems),
    );

    app.add_systems(RunSimulation, next_phase.run_if(in_state(Screen::Gameplay)));

    app.add_systems(RunSimulation, create_water.in_set(PhaseSystems::WaterFlow));
    app.add_systems(
        RunSimulation,
        launch_neutrons.in_set(PhaseSystems::NeutronRelease),
    );
    app.add_systems(
        RunSimulation,
        (vent_steam, turn_uranium_into_xenon, track_cell_pressure)
            .in_set(PhaseSystems::SteamVenting),
    );

    app.add_systems(
        Update,
        handle_overpressure_timer.run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(on_launch_neutron);
}

#[derive(States, Clone, Copy, Reflect, Default, Debug, Eq, PartialEq, Hash)]
pub enum Phase {
    #[default]
    PowerGeneration,
    WaterFlow,
    NeutronRelease,
    SteamVenting,
}

impl Phase {
    fn next(&self) -> Self {
        match *self {
            Self::PowerGeneration => Self::WaterFlow,
            Self::WaterFlow => Self::NeutronRelease,
            Self::NeutronRelease => Self::SteamVenting,
            Self::SteamVenting => Self::PowerGeneration,
        }
    }
}

#[derive(SystemSet, Clone, Copy, Reflect, Debug, Hash, PartialEq, Eq)]
pub enum PhaseSystems {
    PowerGeneration,
    WaterFlow,
    NeutronRelease,
    SteamVenting,
}

fn next_phase(state: Res<State<Phase>>, mut next: ResMut<NextState<Phase>>) {
    next.set(state.next());
}

fn create_water(mut commands: Commands) {
    commands.trigger(CreateWaterParticles(WATER_CREATED_PER_TICK));
}

#[derive(Event, Clone, Copy, Reflect)]
pub struct LaunchNeutron {
    pub origin: Entity,
    pub angle: f32,
}

fn on_launch_neutron(
    trigger: Trigger<LaunchNeutron>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform, With<FuelRod>>,
) -> Result {
    let transform = transforms.get(trigger.origin)?.translation();
    let mut layer_mask = LayerMask::ALL;
    layer_mask.remove(GameLayer::Neutron);
    commands.spawn((
        Name::new("Neutron"),
        Neutron::default(),
        Expiry(Timer::from_seconds(NEUTRON_LIFETIME_SEC, TimerMode::Once)),
        CurrentAngle(trigger.angle),
        Origin(trigger.origin),
        Transform::from_xyz(transform.x, transform.y, 25.),
        RigidBody::Kinematic,
        Collider::circle(NEUTRON_RADIUS),
        CollisionLayers::new(GameLayer::Neutron, layer_mask),
        CollisionEventsEnabled,
    ));
    Ok(())
}

fn launch_neutrons(mut commands: Commands, fuel_rods: Query<(Entity, &FuelRod)>) {
    let mut rng = rand::rng();
    for (entity, &fuel_rod) in &fuel_rods {
        if fuel_rod != FuelRod::Uranium {
            continue;
        }

        for _ in 0..MAX_NEUTRONS_RELEASED_PER_TICK {
            if rng.random_range(0.0..1.0) < NEUTRON_SPAWN_CHANCE {
                commands.trigger(LaunchNeutron {
                    origin: entity,
                    angle: rng.random_range(0.0..TAU),
                });
            }
        }
    }
}

fn turn_uranium_into_xenon(mut commands: Commands, fuel_rods: Query<(Entity, &FuelRod)>) {
    let mut rng = rand::rng();
    for (entity, &fuel_rod) in &fuel_rods {
        if fuel_rod == FuelRod::Xenon {
            continue;
        }

        if rng.random_range(0.0..1.0) < XENON_SPAWN_CHANCE_PER_TICK {
            commands.entity(entity).try_insert(FuelRod::Xenon);
        }
    }
}

fn vent_steam(
    mut commands: Commands,
    steam_particles: Query<(Entity, &GlobalTransform, &Lifetime), With<Particle>>,
    mut cells: Query<(&mut ParticleCount, &Children)>,
    container: Single<(Entity, &GlobalTransform), With<SteamContainer>>,
) {
    let (container_entity, container_transform) = container.into_inner();

    for (mut particle_count, children) in &mut cells {
        let mut particles: Vec<_> = children
            .iter()
            .flat_map(|entity| steam_particles.get(entity))
            .collect();
        particles.sort_by(|(_, _, a), (_, _, b)| {
            b.0.elapsed()
                .partial_cmp(&a.0.elapsed())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let vented_particles = particles.into_iter().take(STEAM_VENTED_PER_TICK);
        for (entity, particle_transform, _) in vented_particles {
            particle_count.decrement(1);

            let new_transform = particle_transform.reparented_to(container_transform);
            commands
                .entity(entity)
                .try_remove::<Lifetime>()
                .try_remove::<TargetAngle>()
                .try_remove::<EasedMotion>()
                .try_remove::<InCell>()
                .try_insert((ChildOf(container_entity), new_transform));
            commands.trigger_targets(
                MoveParticle {
                    to: Vec2::new(0.0, 0.0),
                    stop_current: false,
                },
                entity,
            );
        }
    }
}

#[derive(Component)]
struct OverPressureTimer(Timer);

fn track_cell_pressure(
    mut commands: Commands,
    query: Query<(Entity, &ParticleCount), With<Cell>>,
    energy_container: Single<&ParticleContainer, With<EnergyContainer>>,
) {
    for (entity, particle_count) in &query {
        if particle_count.get() > PRESSURE_EXPLOSION_LEVEL {
            commands.trigger(GameOver {
                cause: GameOverCause::Explosion,
                power_generated: energy_container.count,
            });
        } else if particle_count.get() > PRESSURE_WARN_LEVEL {
            commands
                .entity(entity)
                .try_insert(OverPressureTimer(Timer::from_seconds(
                    1.0,
                    TimerMode::Repeating,
                )));
        } else {
            commands
                .entity(entity)
                .try_remove::<OverPressureTimer>()
                .try_insert(CellColor(Color::from(CELL_COLOR)));
        }
    }
}

fn handle_overpressure_timer(
    mut commands: Commands,
    time: Res<Time>,
    query: Single<(Entity, &mut OverPressureTimer)>,
) {
    let (entity, mut timer) = query.into_inner();

    timer.0.tick(time.delta());

    let t = timer.0.fraction();
    let blend = if t <= 0.5 {
        t / 0.5 // 0.0 -> 1.0
    } else {
        1.0 - ((t - 0.5) / 0.5) // 1.0 -> 0.0
    };
    commands.entity(entity).try_insert(CellColor(
        Color::from(WARNING_COLOR).mix(&CELL_COLOR.into(), blend),
    ));
}
