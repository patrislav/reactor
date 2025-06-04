use std::f32::consts::PI;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};

use crate::{reactorview::ReactorViewRenderLayer, screens::Screen};

use super::crt::{CrtExtension, CrtMaterial};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_screen);
}

fn spawn_screen(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CrtMaterial>>,
) {
    let size = Extent3d {
        width: 1024,
        height: 768,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((
        Name::new("Screen Camera"),
        Camera2d,
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::BLACK.into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
        StateScoped(Screen::Gameplay),
    ));

    let mesh = meshes.add(Plane3d::default().mesh().size(5.12, 3.84));
    let material = materials.add(CrtMaterial {
        base: StandardMaterial {
            reflectance: 0.1,
            ..default()
        },
        extension: CrtExtension {
            image: image_handle,
            noise_amount: 0.2,
            vignette_amount: 0.75,
            aberration_amount: 0.01,
        },
    });

    let mut transform = Transform::from_xyz(0.0, 3.7, -7.5);
    transform.rotate_x(0.5 * PI);
    commands.spawn((
        Name::new("Screen"),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
    ));
}
