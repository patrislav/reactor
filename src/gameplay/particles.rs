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
        (update_eased_movement, update_cell_colors).run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(handle_move_particle)
        .add_observer(handle_create_water_particles)
        .add_observer(handle_flow_water_particles_into_cell)
        .add_observer(handle_boil_water_particle)
        .add_observer(handle_finished_particle_motion);
}

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[require(WaterFlow)]
#[reflect(Component)]
pub struct WaterContainer;

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[reflect(Component)]
pub struct WaterFlow(usize);

impl WaterFlow {
    pub fn get(&self) -> usize {
        self.0
    }

    pub fn decrease(&mut self) {
        if self.0 > 1 {
            self.0 -= 1;
        }
    }

    pub fn increase(&mut self) {
        if self.0 < 50 {
            self.0 += 1;
        }
    }
}

impl Default for WaterFlow {
    fn default() -> Self {
        Self(3)
    }
}

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[reflect(Component)]
pub struct SteamContainer;

#[derive(Component, Copy, Clone, Reflect, Debug)]
#[reflect(Component)]
#[require(PowerDemand, NextPowerDemand)]
pub struct EnergyContainer;

#[derive(Component, Clone, Reflect, Debug)]
#[reflect(Component)]
#[require(Transform, Visibility, ContainerSize)]
pub struct ParticleContainer {
    pub particle: Particle,
    pub count: usize,
}

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

#[derive(Event, Clone, Reflect, Debug)]
pub struct FinishedEasedMotion;

#[derive(Event, Clone, Reflect, Default, Debug)]
pub struct MoveParticle {
    pub to: Vec2,
    pub stop_current: bool,
}

#[derive(Event, Clone, Reflect, Debug)]
pub struct CreateWaterParticles(pub usize);

#[derive(Event, Clone, Reflect, Debug)]
pub struct FlowWaterParticlesIntoCell;

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
    commands.entity(trigger.target()).try_insert(EasedMotion {
        from: *current,
        to: current.with_translation(Vec3::new(dest.x, dest.y, current.translation.z)),
        timer: Timer::from_seconds(1.0, TimerMode::Once),
        easing: EaseFunction::CubicInOut,
    });
    Ok(())
}

fn handle_create_water_particles(
    trigger: Trigger<CreateWaterParticles>,
    mut container: Single<&mut ParticleContainer, With<WaterContainer>>,
) {
    container.count = (container.count + trigger.event().0).clamp(0, MAX_WATER_STORED);
}

fn handle_flow_water_particles_into_cell(
    trigger: Trigger<FlowWaterParticlesIntoCell>,
    mut commands: Commands,
    container: Single<(&mut ParticleContainer, &GlobalTransform, &WaterFlow), With<WaterContainer>>,
    mut cells: Query<(Entity, &GlobalTransform, &mut ParticleCount)>,
) -> Result {
    let (mut container, container_transform, water_flow) = container.into_inner();
    let (cell, cell_transform, mut particle_count) = cells.get_mut(trigger.target())?;

    let particle_transform = cell_transform.affine().inverse() * container_transform.affine();
    let mut particle_transform = Transform::from_matrix(particle_transform.into());
    particle_transform.translation.z = 20.0;

    let count = container.count.min(water_flow.0);
    for _ in 0..count {
        container.count -= 1;
        particle_count.increment(1);

        commands.spawn((
            Name::new("Water particle"),
            Particle::Water(false),
            particle_transform,
            ChildOf(cell),
            TargetAngle::default(),
        ));
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
            commands.trigger_targets(FinishedEasedMotion, entity);
            commands.entity(entity).try_remove::<EasedMotion>();
        }
    }
}

fn handle_boil_water_particle(
    trigger: Trigger<BoilWaterParticle>,
    mut cleanup: EventWriter<Cleanup>,
    mut commands: Commands,
    query: Query<&ChildOf, With<Particle>>,
    mut particle_counts: Query<&mut ParticleCount>,
) -> Result {
    let child_of = query.get(trigger.target())?;
    let mut particle_count = particle_counts.get_mut(child_of.0)?;

    for _ in 0..STEAM_GENERATED_PER_WATER {
        particle_count.increment(1);

        commands
            .entity(trigger.target())
            .clone_and_spawn_with(|config| {
                config.deny::<Particle>().deny::<Name>();
            })
            .try_insert((
                Name::new("Steam particle"),
                Particle::Steam,
                Lifetime(Stopwatch::new()),
            ));
    }

    cleanup.write(Cleanup(trigger.target()));
    particle_count.decrement(1);

    Ok(())
}

fn handle_finished_particle_motion(
    trigger: Trigger<FinishedEasedMotion>,
    mut commands: Commands,
    particles: Query<(&Particle, Option<&InCell>)>,
    steam_container: Single<
        &mut ParticleContainer,
        (With<SteamContainer>, Without<EnergyContainer>),
    >,
    energy_container: Single<
        &mut ParticleContainer,
        (With<EnergyContainer>, Without<SteamContainer>),
    >,
    mut cleanup: EventWriter<Cleanup>,
) {
    if let Ok((&particle, maybe_in_cell)) = particles.get(trigger.target()) {
        if maybe_in_cell.is_none() {
            match particle {
                Particle::Water(_) => {
                    commands.entity(trigger.target()).try_insert(InCell);
                }
                Particle::Steam => {
                    steam_container.into_inner().count += 1;
                    cleanup.write(Cleanup(trigger.target()));
                }
                Particle::Energy => {
                    energy_container.into_inner().count += 1;
                    cleanup.write(Cleanup(trigger.target()));
                }
            }
        }
    }
}

fn update_cell_colors(
    query: Query<(&CellColor, &Children), (With<Cell>, Changed<CellColor>)>,
    mesh_materials: Query<&MeshMaterial2d<ColorMaterial>, With<CellButton>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (color, children) in &query {
        for entity in children {
            if let Ok(mesh_material) = mesh_materials.get(*entity) {
                if let Some(material) = materials.get_mut(mesh_material.id()) {
                    material.color = color.0;
                }
            }
        }
    }
}
