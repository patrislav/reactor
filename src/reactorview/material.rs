use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<EdgeMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct EdgeMaterial {
    #[texture(100)]
    #[sampler(101)]
    pub texture: Handle<Image>,
}

impl Material2d for EdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/edge.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/edge.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
