use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{self, GRAY},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
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
}

#[derive(Component, Clone, Copy, Reflect)]
struct DisplayReactorEdge<const N: usize> {
    sim_edge: Entity,
    nodes: (Entity, Entity),
    coolant_channel: bool,
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

        let mesh = meshes.add(Rectangle::new(EDGE_WIDTH, CELL_SIZE));
        //let material = materials.add(Color::from(css::GRAY));
        let material = materials.add(EdgeMaterial {
            texture: assets.arrow_texture.clone(),
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
                    coolant_channel: maybe_coolant_edge.is_some(),
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
        let dir = (t1 - t2).normalize();
        let angle = Vec2::Y.angle_between(dir);

        *edge_transform = Transform::from_xyz(midpoint.x, midpoint.y, 2.0);
        edge_transform.rotate_z(angle);
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
                let visibility = if (edge.coolant_channel && state.coolant_channels_visible())
                    || state.edges_visible()
                {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                commands.entity(entity).insert(visibility);
            }
        }
    }
}
