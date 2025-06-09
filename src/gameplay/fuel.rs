use bevy::prelude::*;

use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_materials);
}

fn update_materials(
    mut commands: Commands,
    query: Query<(Entity, &FuelRod), Changed<FuelRod>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, fuel_rod) in &query {
        let material = materials.add(match fuel_rod {
            FuelRod::Uranium => URANIUM_COLOR,
            FuelRod::Xenon => XENON_COLOR,
        });
        commands.entity(entity).try_insert(MeshMaterial2d(material));
    }
}
