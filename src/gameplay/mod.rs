use std::f32::consts::PI;

use bevy::{color::palettes::css::WHITE, prelude::*, sprite::AlphaMode2d};

use crate::{PausableSystems, asset_tracking::LoadResource, screens::Screen};

pub mod constants;
pub mod crt;
pub mod neutrons;
pub mod particles;
pub mod schedule;
pub mod simulation;
pub mod types;

pub use constants::*;
pub use crt::*;
pub use neutrons::*;
pub use particles::*;
pub use schedule::*;
pub use simulation::*;
pub use types::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(schedule::plugin);
    app.add_plugins(simulation::plugin);
    app.add_plugins(particles::plugin);
    app.add_plugins(types::plugin);
    app.add_plugins(neutrons::plugin);
    app.add_plugins(CrtPlugin);

    app.init_resource::<GameplayAssets>();
    app.load_resource::<GameplayAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_reactor);
    app.add_systems(
        Update,
        (
            update_particle_target_angles,
            //particle_angular_movement,
            //particle_distance_movement,
            particle_movement,
            neutron_distance_movement,
            scale_movement,
            update_particle_transforms,
            update_neutron_transforms,
            update_cell_transforms,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        (tick_lifetimes, tick_expirations, advance_neutron_lifecycle)
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
    app.add_observer(on_add_reactor_core)
        .add_observer(on_add_particle)
        .add_observer(on_add_neutron);

    app.add_systems(PostUpdate, cleanup_entities);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameplayAssets {
    particle: Handle<Mesh>,

    //#[dependency]
    //music: Handle<AudioSource>,
    #[dependency]
    add_water: Handle<Image>,
}

impl FromWorld for GameplayAssets {
    fn from_world(world: &mut World) -> Self {
        world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            let assets = world.resource::<AssetServer>();
            Self {
                particle: meshes.add(Circle::new(PARTICLE_RADIUS)),
                add_water: assets.load("images/add-water.png"),
            }
        })
    }
}

fn spawn_reactor(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        StateScoped(Screen::Gameplay),
        CrtSettings::default(),
    ));
    commands.spawn((
        Name::new("Reactor Core"),
        ReactorCore::default(),
        Visibility::default(),
        Transform::from_xyz(0., 0., 0.),
        StateScoped(Screen::Gameplay),
    ));
    commands.spawn((
        Name::new("Water container"),
        WaterContainer,
        ParticleContainer::default(),
        ContainerSize(Rect::new(-50., -50., 50., 50.)),
        Transform::from_xyz(-400., -100., 0.),
    ));
    commands.spawn((
        Name::new("Steam container"),
        SteamContainer,
        ParticleContainer::default(),
        ContainerSize(Rect::new(-50., -50., 50., 50.)),
        Transform::from_xyz(-400., 100., 0.),
    ));
}

#[derive(Component)]
struct CellButton(Entity);

fn on_add_reactor_core(
    trigger: Trigger<OnAdd, ReactorCore>,
    core: Single<&ReactorCore>,
    assets: Res<GameplayAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let outer_size = CELL_OUTER_SIZE;
    let mesh = meshes.add(Circle::new(CELL_RADIUS));
    let button_mesh = meshes.add(Rectangle::from_size(Vec2::splat(CELL_RADIUS / 2.0)));

    let mut cells = Vec::new();
    for (index, pos) in core.iter_valid_positions().enumerate() {
        let entity = commands
            .spawn((
                Name::new(format!("Cell {}/{}", pos.x, pos.y)),
                ChildOf(trigger.target()),
                Cell(pos),
                CellIndex(index),
                Reactivity(0.5),
                Transform::from_xyz(
                    (pos.x as f32) * outer_size,
                    (pos.y as f32) * outer_size,
                    8.0,
                ),
                Visibility::Inherited,
            ))
            .id();
        let background = commands
            .spawn((
                Name::new("Background"),
                ChildOf(entity),
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(Color::from(CELL_COLOR))),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::Inherited,
                Pickable::default(),
                CurrentScale::default(),
            ))
            .observe(on_cell_pointer_over)
            .observe(on_cell_pointer_out)
            .id();
        commands
            .spawn((
                ChildOf(background),
                CellButton(entity),
                Mesh2d(button_mesh.clone()),
                MeshMaterial2d(materials.add(ColorMaterial {
                    color: WHITE.with_alpha(0.5).into(),
                    alpha_mode: AlphaMode2d::Blend,
                    texture: Some(assets.add_water.clone()),
                    ..default()
                })),
                Pickable::default(),
                Transform::from_xyz(0.0, -CELL_RADIUS / 3.0, 10.0),
            ))
            .observe(on_click_add_water)
            .observe(on_button_over)
            .observe(on_button_out);
        cells.push((pos, entity));
    }
}

fn on_cell_pointer_over(trigger: Trigger<Pointer<Over>>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(TargetScale(1.25));
}

fn on_cell_pointer_out(trigger: Trigger<Pointer<Out>>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(TargetScale(1.));
}

fn on_click_add_water(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<&CellButton>,
) -> Result {
    let button = query.get(trigger.target())?;
    commands.trigger_targets(FlowWaterParticleIntoCell, button.0);
    Ok(())
}

fn on_button_over(
    trigger: Trigger<Pointer<Over>>,
    query: Query<&MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    let material = query.get(trigger.target())?;
    materials
        .get_mut(material.id())
        .ok_or_else(|| anyhow::format_err!("No material"))?
        .color
        .set_alpha(1.0);
    Ok(())
}

fn on_button_out(
    trigger: Trigger<Pointer<Out>>,
    query: Query<&MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    let material = query.get(trigger.target())?;
    materials
        .get_mut(material.id())
        .ok_or_else(|| anyhow::format_err!("No material"))?
        .color
        .set_alpha(0.5);
    Ok(())
}

fn on_add_particle(
    trigger: Trigger<OnAdd, Particle>,
    particles: Query<&Particle>,
    assets: Res<GameplayAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    let particle = particles.get(trigger.target())?;

    commands.entity(trigger.target()).insert((
        Mesh2d(assets.particle.clone()),
        MeshMaterial2d(materials.add(particle.color())),
        TargetDistance(CELL_RADIUS - (PARTICLE_RADIUS * 2.0)),
        StateScoped(Screen::Gameplay),
        Pickable::IGNORE,
    ));

    Ok(())
}

fn on_add_neutron(
    trigger: Trigger<OnAdd, Neutron>,
    assets: Res<GameplayAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    commands.entity(trigger.target()).insert((
        Mesh2d(assets.particle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: NEUTRON_COLOR.into(),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        StateScoped(Screen::Gameplay),
        Pickable::IGNORE,
    ));
    Ok(())
}

fn update_particle_target_angles(
    mut commands: Commands,
    cells: Query<&Children, (With<Cell>, Changed<ParticleCount>)>,
    particles: Query<(Entity, &TargetAngle), With<Particle>>,
) {
    for children in &cells {
        let mut particles: Vec<_> = children
            .iter()
            .flat_map(|entity| particles.get(entity))
            .collect();

        particles
            .sort_by(|(_, a), (_, b)| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let total_particles = particles.len();
        if total_particles == 0 {
            continue;
        }

        for (i, (entity, _)) in particles.into_iter().enumerate() {
            let angle = i as f32 * (2.0 * PI / total_particles as f32);
            commands.entity(entity).insert(TargetAngle(angle));
        }
    }
}

fn particle_movement(
    mut commands: Commands,
    query: Query<(Entity, &TargetAngle), (With<Particle>, Changed<TargetAngle>)>,
) {
    for (entity, angle) in &query {
        let distance = CELL_RADIUS - PARTICLE_RADIUS;
        let x = distance * angle.0.cos();
        let y = distance * angle.0.sin();

        commands.trigger_targets(
            MoveParticle {
                to: Vec2::new(x, y),
                stop_current: false,
            },
            entity,
        );
    }
}

fn neutron_distance_movement(
    time: Res<Time>,
    mut query: Query<&mut CurrentDistance, With<Neutron>>,
) {
    for mut current in &mut query {
        let step = time.delta_secs() * NEUTRON_SPEED;
        current.0 += step;
    }
}

fn update_particle_transforms(
    mut query: Query<(&mut Transform, &CurrentAngle, &CurrentDistance), With<Particle>>,
) {
    for (mut transform, angle, distance) in &mut query {
        let x = distance.0 * angle.0.cos();
        let y = distance.0 * angle.0.sin();
        transform.translation = Vec3::new(x, y, 20.0);
    }
}

fn update_neutron_transforms(
    mut query: Query<(&mut Transform, &Origin, &CurrentAngle, &CurrentDistance), With<Neutron>>,
    origins: Query<&GlobalTransform, With<Cell>>,
) -> Result {
    for (mut transform, origin, angle, distance) in &mut query {
        let origin = origins.get(origin.0)?.translation();
        let x = origin.x + distance.0 * angle.0.cos();
        let y = origin.y + distance.0 * angle.0.sin();
        transform.translation = Vec3::new(x, y, 25.0);
        transform.scale = Vec3::splat(NEUTRON_RADIUS / PARTICLE_RADIUS);
    }
    Ok(())
}
fn scale_movement(time: Res<Time>, mut query: Query<(&mut CurrentScale, &TargetScale)>) {
    for (mut current, target) in &mut query {
        let diff = target.0 - current.0;
        let step = diff * time.delta_secs() * SCALE_SPEED;
        current.0 += step;
    }
}

fn update_cell_transforms(mut query: Query<(&mut Transform, &CurrentScale)>) {
    for (mut transform, current_scale) in &mut query {
        transform.scale = Vec3::new(current_scale.0, current_scale.0, 1.0);
    }
}

fn tick_expirations(time: Res<Time>, mut query: Query<&mut Expiry>) {
    for mut expiry in &mut query {
        expiry.0.tick(time.delta());
    }
}

fn tick_lifetimes(time: Res<Time>, mut query: Query<&mut Lifetime>) {
    for mut lifetime in &mut query {
        lifetime.0.tick(time.delta());
    }
}

fn advance_neutron_lifecycle(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Neutron,
        &Expiry,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    for (entity, mut neutron, expiry, material) in &mut query {
        match *neutron {
            Neutron::Active => {
                if expiry.0.finished() {
                    *neutron = Neutron::Dying;
                    commands
                        .entity(entity)
                        .insert(Expiry(Timer::from_seconds(0.5, TimerMode::Once)));
                }
            }
            Neutron::Dying => {
                if expiry.0.finished() {
                    commands.entity(entity).despawn();
                } else {
                    let material = materials
                        .get_mut(material.id())
                        .ok_or_else(|| anyhow::format_err!("Material must exist"))?;
                    material.color = material.color.with_alpha(expiry.0.fraction_remaining());
                }
            }
        }
    }
    Ok(())
}

fn cleanup_entities(mut commands: Commands, query: Query<Entity, With<Cleanup>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
