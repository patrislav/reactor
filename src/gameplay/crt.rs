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
    #[uniform(100)]
    pub noise_amount: f32,
    #[uniform(101)]
    pub vignette_amount: f32,

    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec2,
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
