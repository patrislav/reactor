use std::f32::consts::TAU;

use avian2d::prelude::{CollisionEventsEnabled, LayerMask};
use bevy::prelude::*;
use rand::Rng;

use super::*;
use crate::PausableSystems;

pub fn plugin(app: &mut App) {
    app.init_state::<Phase>();
    app.configure_sets(
        RunSimulation,
        (
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
    app.add_systems(RunSimulation, vent_steam.in_set(PhaseSystems::SteamVenting));
}

#[derive(States, Clone, Copy, Reflect, Default, Debug, Eq, PartialEq, Hash)]
pub enum Phase {
    #[default]
    WaterFlow,
    NeutronRelease,
    SteamVenting,
}

impl Phase {
    fn next(&self) -> Self {
        match *self {
            Self::WaterFlow => Self::NeutronRelease,
            Self::NeutronRelease => Self::SteamVenting,
            Self::SteamVenting => Self::WaterFlow,
        }
    }
}

#[derive(SystemSet, Clone, Copy, Reflect, Debug, Hash, PartialEq, Eq)]
pub enum PhaseSystems {
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

fn launch_neutrons(
    mut commands: Commands,
    cells: Query<(Entity, &GlobalTransform, &Reactivity), With<Cell>>,
) {
    let mut rng = rand::rng();
    for (entity, transform, reactivity) in &cells {
        // TODO: more than one neutron should be launched
        let cell = transform.translation();
        if reactivity.0 > rng.random_range(0.0..1.0) {
            let mut layer_mask = LayerMask::ALL;
            layer_mask.remove(GameLayer::Neutron);

            commands.spawn((
                Name::new("Neutron"),
                Neutron::default(),
                Expiry(Timer::from_seconds(NEUTRON_LIFETIME_SEC, TimerMode::Once)),
                CurrentAngle(rng.random_range(0.0..TAU)),
                Origin(entity),
                Transform::from_xyz(cell.x, cell.y, 25.),
                RigidBody::Kinematic,
                Collider::circle(NEUTRON_RADIUS),
                CollisionLayers::new(GameLayer::Neutron, layer_mask),
                CollisionEventsEnabled,
            ));
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
            if particle_count.0 > 0 {
                particle_count.0 -= 1;
            }

            let new_transform = particle_transform.reparented_to(container_transform);
            commands
                .entity(entity)
                .remove::<Lifetime>()
                .remove::<TargetAngle>()
                .remove::<EasedMotion>()
                .remove::<InCell>()
                .insert((ChildOf(container_entity), new_transform));
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
