use bevy::{platform::collections::HashSet, prelude::*};

use crate::{PausableSystems, screens::Screen};

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        check_neutron_collisions
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

#[derive(Event, Reflect, Copy, Clone, Debug)]
pub struct BoilWaterParticle;

fn check_neutron_collisions(
    mut commands: Commands,
    neutrons: Query<(Entity, &GlobalTransform), Without<Particle>>,
    particles: Query<
        (Entity, &Particle, &GlobalTransform, Option<&EasedMotion>),
        (Without<Neutron>, With<TargetAngle>),
    >,
) {
    let mut hit_particles = HashSet::new();
    for (neutron, neutron_transform) in &neutrons {
        'inner: for (particle_entity, &particle, particle_transform, maybe_eased_motion) in
            &particles
        {
            if particle != Particle::Water {
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
                commands.trigger_targets(BoilWaterParticle, particle_entity);
                commands.entity(neutron).insert(Cleanup);
                hit_particles.insert(particle_entity);
                break 'inner;
            }
        }
    }
}
