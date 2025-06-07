use bevy::prelude::*;

use crate::simulation::{
    events::{PropertyChanged, RequestProperty},
    types::ReactorCell,
};

pub fn plugin(app: &mut App) {
    app.register_type::<ControlRodStatus>();
    app.register_type::<VariableGetter>();
    app.register_type::<VariableColorMaterial>();
    app.register_type::<VariableEmissiveMaterial>();
    app.register_type::<Emissive>();
    app.register_type::<VariableStandardMaterial>();
    app.register_type::<Option<LinearRgba>>();
    app.register_type::<Option<StandardMaterial>>();
    app.register_type::<InterpolateTransform>();

    app.add_event::<PropertyChanged<ReactorCell, ControlRodStatus>>();
    app.add_event::<RequestProperty<ReactorCell, ControlRodStatus>>();

    app.add_observer(on_add_variable_dependent::<VariableColorMaterial>);
    app.add_observer(on_add_variable_dependent::<VariableEmissiveMaterial>);
    app.add_observer(on_add_variable_dependent::<VariableStandardMaterial>);
    app.add_observer(on_add_variable_dependent::<InterpolateTransform>);

    app.add_systems(Update, update_color_materials);
    app.add_systems(Update, update_emissive_materials);
    app.add_systems(Update, interpolate_transforms);
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
enum VariableGetter {
    #[default]
    None,
    ControlRodStatus(usize),
    ValveStatus(usize),
    PumpPowerLevel(usize),
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
struct VariableValue<T>(T);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
struct VariableGetterLink(Entity);

#[derive(Clone, Copy, Default, Reflect, Eq, PartialEq, Hash)]
pub enum ControlRodStatus {
    #[default]
    Default,
    Selected,
    Malfunctioning,
}

impl From<ControlRodStatus> for usize {
    fn from(value: ControlRodStatus) -> Self {
        match value {
            ControlRodStatus::Default => 0,
            ControlRodStatus::Selected => 1,
            ControlRodStatus::Malfunctioning => 2,
        }
    }
}

/*
#[derive(Event, Clone, Copy, Reflect)]
pub struct PumpStatusChanged {
    pump_index: usize,
    status: PumpStatus,
}
*/

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
struct VariableColorMaterial {
    color0: Option<LinearRgba>,
    color1: Option<LinearRgba>,
    color2: Option<LinearRgba>,
    color3: Option<LinearRgba>,
}

impl VariableColorMaterial {
    fn prop(&self, index: usize) -> Option<&LinearRgba> {
        match index {
            0 => self.color0.as_ref(),
            1 => self.color1.as_ref(),
            2 => self.color2.as_ref(),
            3 => self.color3.as_ref(),
            _ => None,
        }
    }
}

#[derive(Clone, Default, Reflect)]
struct Emissive {
    base_color: LinearRgba,
    emissive: LinearRgba,
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
struct VariableEmissiveMaterial {
    color0: Option<Emissive>,
    color1: Option<Emissive>,
    color2: Option<Emissive>,
    color3: Option<Emissive>,
}

impl VariableEmissiveMaterial {
    fn prop(&self, index: usize) -> Option<&Emissive> {
        match index {
            0 => self.color0.as_ref(),
            1 => self.color1.as_ref(),
            2 => self.color2.as_ref(),
            3 => self.color3.as_ref(),
            _ => None,
        }
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
struct VariableStandardMaterial {
    color0: Option<StandardMaterial>,
    color1: Option<StandardMaterial>,
    color2: Option<StandardMaterial>,
    color3: Option<StandardMaterial>,
}

impl VariableStandardMaterial {
    fn prop(&self, index: usize) -> Option<&StandardMaterial> {
        match index {
            0 => self.color0.as_ref(),
            1 => self.color1.as_ref(),
            2 => self.color2.as_ref(),
            3 => self.color3.as_ref(),
            _ => None,
        }
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
struct InterpolateTransform {
    from: Transform,
    to: Transform,
}

impl InterpolateTransform {
    fn interpolate(&self, s: f32) -> Transform {
        let s = s.clamp(0.0, 1.0);
        Transform {
            translation: self.from.translation.lerp(self.to.translation, s),
            rotation: self.from.rotation.slerp(self.to.rotation, s),
            scale: self.from.scale.lerp(self.to.scale, s),
        }
    }
}

fn on_add_variable_dependent<T>(
    trigger: Trigger<OnAdd, T>,
    mut commands: Commands,
    child_of: Query<&ChildOf>,
    variable_getters: Query<(Entity, &VariableGetter)>,
) -> Result
where
    T: Component,
{
    let var_dep_entity = trigger.target();
    let (entity, getter) = variable_getters.get(var_dep_entity).or_else(|_| {
        child_of
            .iter_ancestors(var_dep_entity)
            .find_map(|ancestor| variable_getters.get(ancestor).ok())
            .ok_or_else(|| anyhow::format_err!("Entity {} has no VariableGetter", trigger.target()))
    })?;

    match getter {
        VariableGetter::None => {}
        VariableGetter::ControlRodStatus(cell_index) => {
            info!("Adding property observer of ControlRodStatus for {cell_index}");
            add_property_observer::<ReactorCell, ControlRodStatus, usize>(
                &mut commands,
                entity,
                var_dep_entity,
                *cell_index,
            );
        }
        _ => {}
    }

    commands
        .entity(trigger.target())
        .insert(VariableGetterLink(entity));

    Ok(())
}

fn add_property_observer<T, P, V>(
    commands: &mut Commands,
    getter_entity: Entity,
    var_dep_entity: Entity,
    index: usize,
) where
    T: Send + Sync + 'static,
    V: Send + Sync + 'static,
    P: Into<V> + Send + Sync + Copy + 'static,
{
    commands.spawn((
        ChildOf(getter_entity),
        Observer::new(
            move |trigger: Trigger<PropertyChanged<T, P>>, mut commands: Commands| {
                if trigger.event().index != index {
                    return;
                }

                info!("Property changed!");

                let value: V = trigger.event().value.into();
                commands.entity(var_dep_entity).insert(VariableValue(value));
            },
        ),
    ));
    commands.trigger(RequestProperty::<T, P>::new(index));
}

fn update_color_materials(
    mut commands: Commands,
    query: Query<
        (Entity, &VariableColorMaterial, &VariableValue<usize>),
        Changed<VariableValue<usize>>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, var_material, var_value) in &query {
        if let Some(color) = var_material.prop(var_value.0) {
            let material = materials.add(StandardMaterial {
                base_color: (*color).into(),
                ..default()
            });
            commands.entity(entity).insert(MeshMaterial3d(material));
        }
    }
}

fn update_emissive_materials(
    mut commands: Commands,
    query: Query<
        (Entity, &VariableEmissiveMaterial, &VariableValue<usize>),
        Changed<VariableValue<usize>>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, var_material, var_value) in &query {
        info!("Changing material of {entity} based on VariableValue");
        if let Some(color) = var_material.prop(var_value.0) {
            let material = materials.add(StandardMaterial {
                base_color: color.base_color.into(),
                emissive: color.emissive,
                ..default()
            });
            commands.entity(entity).insert(MeshMaterial3d(material));
        }
    }
}

fn update_standard_materials(
    mut commands: Commands,
    query: Query<
        (Entity, &VariableStandardMaterial, &VariableValue<usize>),
        Changed<VariableValue<usize>>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, var_material, var_value) in &query {
        if let Some(material) = var_material.prop(var_value.0) {
            let material = materials.add(material.clone());
            commands.entity(entity).insert(MeshMaterial3d(material));
        }
    }
}

fn interpolate_transforms(
    mut commands: Commands,
    query: Query<(Entity, &InterpolateTransform, &VariableValue<f32>), Changed<VariableValue<f32>>>,
) {
    for (entity, interpolate_transform, value) in &query {
        commands
            .entity(entity)
            .insert(interpolate_transform.interpolate(value.0));
    }
}
