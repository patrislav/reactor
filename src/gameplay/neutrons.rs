use avian2d::prelude::CollisionStarted;
use bevy::{platform::collections::HashSet, prelude::*};
use rand::Rng;

use crate::{PausableSystems, screens::Screen};

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            check_neutron_collisions_with_particles,
            check_neutron_out_of_bounds,
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
    app.add_systems(Update, handle_neutron_collisions);
}

#[derive(Event, Reflect, Copy, Clone, Debug)]
pub struct BoilWaterParticle;

#[derive(Event, Reflect, Copy, Clone, Debug)]
pub struct NeutronCollision;

fn handle_neutron_collisions(
    mut events: EventReader<CollisionStarted>,
    mut commands: Commands,
    neutrons: Query<(&CurrentAngle, &Origin), With<Neutron>>,
    control_rods: Query<&ControlRodInsertion, With<ControlRod>>,
    fuel_rods: Query<&FuelRod>,
    mut cleanup: EventWriter<Cleanup>,
) {
    for CollisionStarted(entity1, entity2) in events.read() {
        let ((neutron_angle, neutron_origin), neutron_entity, other_entity) =
            if let Ok(neutron) = neutrons.get(*entity1) {
                (neutron, *entity1, *entity2)
            } else if let Ok(neutron) = neutrons.get(*entity2) {
                (neutron, *entity2, *entity1)
            } else {
                continue;
            };

        if let Ok(insertion) = control_rods.get(other_entity) {
            let mut rng = rand::rng();
            let chance = rng.random_range(0.0..1.0);
            if chance < insertion.0 {
                cleanup.write(Cleanup(neutron_entity));
            }
        } else if let Ok(fuel_rod) = fuel_rods.get(other_entity) {
            if neutron_origin.0 == other_entity {
                continue;
            }

            commands.trigger_targets(NeutronCollision, neutron_entity);
            match fuel_rod {
                FuelRod::Uranium => {
                    let angles = [-0.2 * PI, 0.0, 0.2 * PI];
                    for angle in angles {
                        commands.trigger(LaunchNeutron {
                            origin: other_entity,
                            angle: neutron_angle.0 + angle,
                        });
                    }
                    cleanup.write(Cleanup(neutron_entity));
                }
                FuelRod::Xenon => {
                    commands.entity(other_entity).try_insert(FuelRod::Uranium);
                    cleanup.write(Cleanup(neutron_entity));
                }
            }
        }
    }
}

fn check_neutron_collisions_with_particles(
    mut commands: Commands,
    neutrons: Query<(Entity, &GlobalTransform), (With<Neutron>, Without<Particle>)>,
    mut particles: Query<
        (
            Entity,
            &mut Particle,
            &GlobalTransform,
            Option<&EasedMotion>,
        ),
        (Without<Neutron>, With<InCell>),
    >,
    mut cleanup: EventWriter<Cleanup>,
) {
    let mut hit_particles = HashSet::new();
    for (neutron, neutron_transform) in &neutrons {
        'inner: for (particle_entity, mut particle, particle_transform, maybe_eased_motion) in
            &mut particles
        {
            if *particle != Particle::Water(false) {
                continue;
            }
            if let Some(eased_motion) = maybe_eased_motion {
                if eased_motion.timer.fraction_remaining() > 0.1 {
                    continue;
                }
            }

            let distance = neutron_transform
                .translation()
                .xy()
                .distance(particle_transform.translation().xy());
            if distance < NEUTRON_RADIUS + PARTICLE_RADIUS + COLLISION_LEEWAY
                && !hit_particles.contains(&particle_entity)
            {
                *particle = Particle::Water(true);
                commands.trigger_targets(BoilWaterParticle, particle_entity);
                cleanup.write(Cleanup(neutron));
                hit_particles.insert(particle_entity);
                break 'inner;
            }
        }
    }
}

fn check_neutron_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Neutron, &GlobalTransform), With<Neutron>>,
) {
    for (entity, neutron, transform) in &query {
        if *neutron == Neutron::Dying {
            continue;
        }

        let w = CELL_OUTER_SIZE * (CELL_COLUMNS as f32);
        let h = CELL_OUTER_SIZE * (CELL_ROWS as f32);
        let rect = Rect::from_corners(Vec2::new(-w / 2., -h / 2.), Vec2::new(w / 2., h / 2.));
        if !rect.contains(transform.translation().xy()) {
            commands.entity(entity).try_insert((
                Neutron::Dying,
                Expiry(Timer::from_seconds(0.5, TimerMode::Once)),
            ));
        }
    }
}
