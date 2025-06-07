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
    pub mask_texture: Handle<Image>,

    #[uniform(102)]
    pub scroll_speed: Vec2,

    #[uniform(103)]
    pub bg_color: LinearRgba,

    #[uniform(104)]
    pub fg_color: LinearRgba,

    #[uniform(105)]
    pub base_color: LinearRgba,

    #[uniform(106)]
    pub intensity: f32,
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
