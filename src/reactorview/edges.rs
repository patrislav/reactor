use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{self, GRAY},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexAttributeId, MeshVertexBufferLayoutRef},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

use super::material::EdgeMaterial;

use crate::{screens::Screen, simulation::types::*};

use super::ReactorViewAssets;
use super::view::{CELL_SIZE, DisplayMode, EDGE_WIDTH, Grid, ReactorCellLink};

pub(super) fn plugin<const N: usize>(app: &mut App) {
    app.register_type::<DisplayReactorEdge<N>>();
    app.register_type::<ReactorEdgeDisplayLink<N>>();

    app.add_systems(Update, sync_reactor_edges_with_display::<N>);
    app.add_systems(Update, sync_display_edge_positions::<N>);
    app.add_systems(Update, sync_display_edge_visibility::<N>);
    app.add_systems(
        Update,
        sync_display_edge_temperature_style::<N>
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(DisplayMode::<N>::Temperature)),
    );
    app.add_systems(
        Update,
        sync_display_edge_cooling_style::<N>
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(DisplayMode::<N>::Cooling)),
    );
}

#[derive(Component, Clone, Copy, Reflect)]
pub struct DisplayReactorEdge<const N: usize> {
    pub sim_edge: Entity,
    pub nodes: (Entity, Entity),
    pub valve: Option<Entity>,
}

#[derive(Component, Clone, Copy, Reflect)]
struct ReactorEdgeDisplayLink<const N: usize>(Entity);

fn sync_reactor_edges_with_display<const N: usize>(
    mut commands: Commands,
    assets: Res<ReactorViewAssets>,
    edges: Query<
        (Entity, &ReactorEdge, Option<&ReactorCoolantEdge>, &Name),
        Without<ReactorEdgeDisplayLink<N>>,
    >,
    core: Single<&ReactorCore>,
    grid: Single<(Entity, &Grid<N>, &RenderLayers)>,
    display_cells: Query<&GlobalTransform, With<ReactorCellLink<N>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials: ResMut<Assets<EdgeMaterial>>,
) -> Result {
    let (grid_entity, grid, render_layers) = grid.into_inner();
    for (edge_entity, edge, maybe_coolant_edge, edge_name) in &edges {
        let first_cell = core
            .cells_by_pos
            .get(&edge.nodes.0)
            .ok_or(anyhow::format_err!("No cell found"))?;
        let second_cell = core
            .cells_by_pos
            .get(&edge.nodes.1)
            .ok_or(anyhow::format_err!("No cell found"))?;

        let first_display_cell = *grid
            .simulation_to_display_cells
            .get(first_cell)
            .ok_or(anyhow::format_err!("No display cell found"))?;
        let second_display_cell = *grid
            .simulation_to_display_cells
            .get(second_cell)
            .ok_or(anyhow::format_err!("No display cell found"))?;

        let first_transform = display_cells.get(first_display_cell)?;
        let second_transform = display_cells.get(second_display_cell)?;

        let mut mesh: Mesh = Rectangle::new(EDGE_WIDTH, CELL_SIZE).into();
        let uv_extent = CELL_SIZE / EDGE_WIDTH;
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[uv_extent, 1.0], [uv_extent, 0.0], [0.0, 0.0], [0.0, 1.0]],
        );
        let mesh = meshes.add(mesh);
        //let material = materials.add(Color::from(css::GRAY));
        let material = materials.add(EdgeMaterial {
            mask_texture: assets.arrow_texture.clone(),
            scroll_speed: Vec2::new(-1.0, 0.0),
            bg_color: LinearRgba::rgb(0.5, 0.5, 0.5),
            fg_color: LinearRgba::rgb(1.0, 0.0, 0.0),
            base_color: css::GRAY.into(),
            intensity: 0.0,
        });

        let midpoint =
            (first_transform.translation().xy() + second_transform.translation().xy()) * 0.5;
        let display_entity = commands
            .spawn((
                Name::new(format!("Display for {}", edge_name)),
                ChildOf(grid_entity),
                DisplayReactorEdge::<N> {
                    sim_edge: edge_entity,
                    nodes: (first_display_cell, second_display_cell),
                    valve: maybe_coolant_edge.map(|coolant_edge| coolant_edge.0),
                },
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(midpoint.x, midpoint.y, 2.0),
                render_layers.clone(),
            ))
            .id();
        debug!("Spawned view edge: {:?}", display_entity);
        commands
            .entity(edge_entity)
            .insert(ReactorEdgeDisplayLink::<N>(display_entity));
    }

    Ok(())
}

fn sync_display_edge_positions<const N: usize>(
    mut edges: Query<(&mut Transform, &DisplayReactorEdge<N>)>,
    cells: Query<&GlobalTransform, With<ReactorCellLink<N>>>,
) -> Result {
    for (mut edge_transform, edge) in &mut edges {
        let t1 = cells.get(edge.nodes.0)?.translation().xy();
        let t2 = cells.get(edge.nodes.1)?.translation().xy();

        let midpoint = (t1 + t2) * 0.5;
        edge_transform.translation = Vec3::new(midpoint.x, midpoint.y, 2.0);
    }

    Ok(())
}

fn sync_display_edge_visibility<const N: usize>(
    mut commands: Commands,
    mut events: EventReader<StateTransitionEvent<DisplayMode<N>>>,
    edges: Query<(Entity, &DisplayReactorEdge<N>)>,
) {
    for event in events.read() {
        for (entity, edge) in &edges {
            if let Some(state) = event.entered {
                let visibility = if state.edge_visible(edge) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                commands.entity(entity).insert(visibility);
            }
        }
    }
}

fn sync_display_edge_temperature_style<const N: usize>(
    mut edges: Query<(
        &mut Transform,
        &DisplayReactorEdge<N>,
        &MeshMaterial2d<EdgeMaterial>,
    )>,
    display_cells: Query<(&GlobalTransform, &ReactorCellLink<N>)>,
    cells: Query<&Temperature, With<ReactorCell>>,
    mut materials: ResMut<Assets<EdgeMaterial>>,
) -> Result {
    for (mut edge_transform, edge, material_handle) in &mut edges {
        let (transform1, link1) = display_cells.get(edge.nodes.0)?;
        let (transform2, link2) = display_cells.get(edge.nodes.1)?;
        let temp1 = cells.get(link1.0)?;
        let temp2 = cells.get(link2.0)?;

        let temp_diff = (temp1.0 - temp2.0).abs();
        let dir = (transform1.translation().xy() - transform2.translation().xy()).normalize();
        let angle = if temp1.0 < temp2.0 {
            Vec2::Y.angle_to(dir)
        } else {
            Vec2::Y.angle_to(-dir)
        };

        edge_transform.rotation = Quat::IDENTITY;
        edge_transform.rotate_z(angle);

        let material = materials
            .get_mut(material_handle)
            .ok_or_else(|| anyhow::format_err!("No material found"))?;

        let intensity = (temp_diff / 200.).clamp(0.0, 1.0);
        material.intensity = intensity;
        material.scroll_speed = Vec2::new(-intensity * 5.0, 0.0);
        material.bg_color = LinearRgba::rgb(1.0, 0.2, 0.2);
        material.fg_color = LinearRgba::rgb(0.5, 0.0, 0.0);
    }

    Ok(())
}

fn sync_display_edge_cooling_style<const N: usize>(
    mut edges: Query<(
        &mut Transform,
        &DisplayReactorEdge<N>,
        &MeshMaterial2d<EdgeMaterial>,
    )>,
    display_cells: Query<&GlobalTransform, With<ReactorCellLink<N>>>,
    valves: Query<(&CoolantValve, &ChildOf)>,
    circuits: Query<&CoolantCircuit>,
    mut materials: ResMut<Assets<EdgeMaterial>>,
) -> Result {
    for (mut edge_transform, edge, material_handle) in &mut edges {
        let Some(valve_entity) = edge.valve else {
            continue;
        };
        let (valve, circuit_entity) = valves
            .get(valve_entity)
            .map(|(valve, child_of)| (valve, child_of.0))?;
        let circuit = circuits.get(circuit_entity)?;

        let transform1 = display_cells.get(edge.nodes.0)?;
        let transform2 = display_cells.get(edge.nodes.1)?;
        let dir = (transform1.translation().xy() - transform2.translation().xy()).normalize();
        let angle = Vec2::Y.angle_to(-dir);

        edge_transform.rotation = Quat::IDENTITY;
        edge_transform.rotate_z(angle);

        let material = materials
            .get_mut(material_handle)
            .ok_or_else(|| anyhow::format_err!("No material found"))?;

        let intensity = if valve.open {
            circuit.power.clamp(0.0, 1.0)
        } else {
            0.0
        };
        material.intensity = intensity;
        material.scroll_speed = Vec2::new(-intensity * 5.0, 0.0);
        material.bg_color = LinearRgba::rgb(0.2, 0.2, 1.0);
        material.fg_color = LinearRgba::rgb(0.0, 0.0, 0.5);
    }

    Ok(())
}
