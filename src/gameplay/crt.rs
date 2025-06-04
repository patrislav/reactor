use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<CrtMaterial>::default());
}

pub type CrtMaterial = ExtendedMaterial<StandardMaterial, CrtExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CrtExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[texture(100)]
    #[sampler(101)]
    pub image: Handle<Image>,

    #[uniform(102)]
    pub noise_amount: f32,
    #[uniform(103)]
    pub vignette_amount: f32,
    #[uniform(104)]
    pub aberration_amount: f32,
}

const SHADER_ASSET_PATH: &str = "shaders/crt.wgsl";

impl MaterialExtension for CrtExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
