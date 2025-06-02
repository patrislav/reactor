use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{self, GRAY},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use crate::{screens::Screen, simulation::types::*};

use super::{CELL_SIZE, DisplayMode, EDGE_WIDTH, Grid, ReactorCellLink, ReactorViewRenderLayer};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, sync_reactor_edges_with_display);
    app.add_systems(Update, sync_display_edge_positions);
    app.add_systems(Update, sync_display_edge_visibility);
}

#[derive(Component, Clone, Copy, Reflect)]
struct DisplayReactorEdge {
    sim_edge: Entity,
    nodes: (Entity, Entity),
}

#[derive(Component, Clone, Copy, Reflect)]
struct ReactorEdgeDisplayLink(Entity);

fn sync_reactor_edges_with_display(
    mut commands: Commands,
    screen: Res<State<Screen>>,
    render_layer: Res<ReactorViewRenderLayer>,
    edges: Query<(Entity, &ReactorEdge, &Name), Without<ReactorEdgeDisplayLink>>,
    core: Single<&ReactorCore>,
    grid: Single<&Grid>,
    display_cells: Query<&GlobalTransform, With<ReactorCellLink>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result {
    for (edge_entity, edge, edge_name) in &edges {
        let first_cell = core
            .cells
            .get(&edge.nodes.0)
            .ok_or(anyhow::format_err!("No cell found"))?;
        let second_cell = core
            .cells
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
        let material = materials.add(Color::from(css::GRAY));

        let midpoint =
            (first_transform.translation().xy() + second_transform.translation().xy()) * 0.5;
        let display_entity = commands
            .spawn((
                Name::new(format!("Display for {}", edge_name)),
                DisplayReactorEdge {
                    sim_edge: edge_entity,
                    nodes: (first_display_cell, second_display_cell),
                },
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(midpoint.x, midpoint.y, 2.0),
                render_layer.0.clone(),
                StateScoped(*screen.get()),
            ))
            .id();
        commands
            .entity(edge_entity)
            .insert(ReactorEdgeDisplayLink(display_entity));
    }

    Ok(())
}

fn sync_display_edge_positions(
    mut edges: Query<(&mut Transform, &DisplayReactorEdge)>,
    cells: Query<&GlobalTransform, With<ReactorCellLink>>,
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

fn sync_display_edge_visibility(
    mut commands: Commands,
    mut events: EventReader<StateTransitionEvent<DisplayMode>>,
    edges: Query<Entity, With<DisplayReactorEdge>>,
) {
    for event in events.read() {
        for edge in &edges {
            if let Some(state) = event.entered {
                let visibility = match state.edges_visible() {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                };
                commands.entity(edge).insert(visibility);
            }
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct EdgeMaterial {}

impl Material2d for EdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/edge.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/edge.wgsl".into()
    }
}
