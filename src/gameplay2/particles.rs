use bevy::{prelude::*, time::Stopwatch};

use super::*;

pub fn plugin(app: &mut App) {
    app.register_type::<WaterContainer>();
    app.register_type::<SteamContainer>();
    app.register_type::<ParticleContainer>();
    app.register_type::<ContainerSize>();
    app.register_type::<EasedMotion>();

    app.add_systems(
        Update,
        (update_target_positions, update_eased_movement).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(handle_move_particle)
        .add_observer(handle_create_water_particle)
        .add_observer(handle_flow_water_particle_into_cell)
        .add_observer(handle_boil_water_particle);
}

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[reflect(Component)]
#[require(ParticleContainer)]
pub struct WaterContainer;

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[reflect(Component)]
#[require(ParticleContainer)]
pub struct SteamContainer;

#[derive(Component, Clone, Reflect, Debug, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, ContainerSize)]
pub struct ParticleContainer(pub Vec<Entity>);

#[derive(Component, Copy, Clone, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct ContainerSize(pub Rect);

#[derive(Component, Clone, Reflect, Debug)]
#[reflect(Component)]
pub struct EasedMotion {
    pub from: Transform,
    pub to: Transform,
    pub timer: Timer,
    pub easing: EaseFunction,
}

#[derive(Event, Clone, Reflect, Default, Debug)]
pub struct MoveParticle {
    pub to: Vec2,
    pub stop_current: bool,
}

#[derive(Event, Clone, Reflect, Debug)]
pub struct CreateWaterParticle;

#[derive(Event, Clone, Reflect, Debug)]
pub struct FlowWaterParticleIntoCell;

#[derive(Event, Clone, Reflect, Debug)]
pub struct VentSteamParticleFromCell;

fn handle_move_particle(
    trigger: Trigger<MoveParticle>,
    mut commands: Commands,
    mut particles: Query<(&Transform, Option<&mut EasedMotion>), With<Particle>>,
) -> Result {
    let (current, maybe_motion) = particles.get_mut(trigger.target())?;
    let (dest, update_current) = (trigger.event().to, !trigger.event().stop_current);
    if let Some(mut motion) = maybe_motion {
        if update_current && motion.timer.fraction_remaining() > 0.1 {
            motion.to = current.with_translation(Vec3::new(dest.x, dest.y, current.translation.z));
            return Ok(());
        }
    }
    commands.entity(trigger.target()).insert(EasedMotion {
        from: *current,
        to: current.with_translation(Vec3::new(dest.x, dest.y, current.translation.z)),
        timer: Timer::from_seconds(1.0, TimerMode::Once),
        easing: EaseFunction::CubicInOut,
    });
    Ok(())
}

fn handle_create_water_particle(
    _trigger: Trigger<CreateWaterParticle>,
    mut commands: Commands,
    container: Single<(Entity, &mut ParticleContainer), With<WaterContainer>>,
) {
    let (container_entity, mut queue) = container.into_inner();
    let entity = commands
        .spawn((
            Particle::Water,
            ChildOf(container_entity),
            Transform::from_xyz(0.0, 0.0, 20.0),
        ))
        .id();
    queue.0.insert(0, entity);
}

fn handle_flow_water_particle_into_cell(
    trigger: Trigger<FlowWaterParticleIntoCell>,
    mut commands: Commands,
    container: Single<&mut ParticleContainer, With<WaterContainer>>,
    mut cells: Query<(Entity, &GlobalTransform, &mut ParticleCount)>,
    mut particles: Query<&GlobalTransform>,
) -> Result {
    let (cell, cell_transform, mut particle_count) = cells.get_mut(trigger.target())?;
    if let Some(entity) = container.into_inner().0.pop() {
        let particle_transform = particles.get_mut(entity)?;
        let new_transform = particle_transform.reparented_to(cell_transform);

        commands
            .entity(entity)
            .insert((new_transform, ChildOf(cell), TargetAngle::default()))
            .remove::<EasedMotion>();
        particle_count.0 += 1;
    }

    Ok(())
}

fn update_target_positions(
    mut commands: Commands,
    containers: Query<(&ParticleContainer, &ContainerSize), Changed<ParticleContainer>>,
) -> Result {
    for (container, size) in &containers {
        let (width, height) = (size.0.width(), size.0.height());
        let origin = Vec2::new(-width / 2.0, -height / 2.0);
        let row_capacity = (width / (PARTICLE_RADIUS + CONTAINER_SPACING)).floor() as usize;
        for (index, &entity) in container.0.iter().enumerate() {
            let col = index % row_capacity;
            let row = index / row_capacity;

            let x = col as f32 * (PARTICLE_RADIUS * 2.0 + CONTAINER_SPACING);
            let y = -(row as f32) * (PARTICLE_RADIUS * 2.0 + CONTAINER_SPACING);

            commands.trigger_targets(
                MoveParticle {
                    to: origin + Vec2::new(x, y),
                    stop_current: false,
                },
                entity,
            );
        }
    }
    Ok(())
}

fn update_eased_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut EasedMotion)>,
) {
    for (entity, mut transform, mut motion) in &mut query {
        motion.timer.tick(time.delta());

        let s = motion.easing.sample_clamped(motion.timer.fraction());
        *transform = Transform {
            translation: motion.from.translation.lerp(motion.to.translation, s),
            rotation: motion.from.rotation.slerp(motion.to.rotation, s),
            scale: motion.from.scale.lerp(motion.to.scale, s),
        };

        if motion.timer.just_finished() {
            commands.entity(entity).remove::<EasedMotion>();
        }
    }
}

fn handle_boil_water_particle(
    trigger: Trigger<BoilWaterParticle>,
    mut commands: Commands,
    query: Query<&ChildOf, With<Particle>>,
    mut particle_counts: Query<&mut ParticleCount>,
) -> Result {
    let child_of = query.get(trigger.target())?;
    let mut particle_count = particle_counts.get_mut(child_of.0)?;

    for _ in 0..STEAM_GENERATED_PER_WATER {
        commands
            .entity(trigger.target())
            .clone_and_spawn_with(|config| {
                config.deny::<Particle>();
            })
            .insert((Particle::Steam, Lifetime(Stopwatch::new())));
    }
    commands.entity(trigger.target()).insert(Cleanup);

    particle_count.0 += STEAM_GENERATED_PER_WATER - 1;
    Ok(())
}
