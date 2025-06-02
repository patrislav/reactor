use avian3d::prelude::*;
use bevy::prelude::*;

use super::{
    //physics::GameLayer,
    player::{Player, PlayerCamera},
};

pub fn plugin(app: &mut App) {
    app.register_type::<Interactable>()
        .register_type::<InteractableAnimationTarget>()
        .add_systems(Update, update_interactable_animation_targets)
        .add_systems(Update, update_interaction_candidate)
        .add_observer(on_add_interactable_animation_target)
        .add_observer(on_add_interactable);
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
pub struct InteractionCandidate(pub Option<Entity>);

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component, Default)]
pub enum Interactable {
    Button(bool),
    Lever(f32),
}

impl Default for Interactable {
    fn default() -> Self {
        Self::Button(false)
    }
}

fn on_add_interactable(trigger: Trigger<OnAdd, Interactable>, mut commands: Commands) {
    commands.entity(trigger.target()).insert((
        RigidBody::Static,
        //CollisionLayers::new(GameLayer::Interactable, LayerMask(0)),
        Sensor,
    ));
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
pub struct InteractableAnimationTarget {
    pub from_transform: Transform,
    pub to_transform: Transform,
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
struct InteractableAnimationTargetLink(Entity);

fn on_add_interactable_animation_target(
    trigger: Trigger<OnAdd, InteractableAnimationTarget>,
    mut commands: Commands,
    children_q: Query<&ChildOf>,
    interactables_q: Query<(), With<Interactable>>,
) {
    let ancestor = children_q
        .iter_ancestors(trigger.target())
        .find(|entity| interactables_q.contains(*entity));
    if let Some(ancestor) = ancestor {
        commands
            .entity(ancestor)
            .insert(InteractableAnimationTargetLink(trigger.target()));
    }
}

fn update_interactable_animation_targets(
    interactables: Query<(&Interactable, &InteractableAnimationTargetLink), Changed<Interactable>>,
    mut animation_targets: Query<(&InteractableAnimationTarget, &mut Transform)>,
) {
    for (interactable, link) in &interactables {
        if let Ok((anim_target, mut transform)) = animation_targets.get_mut(link.0) {
            *transform = match interactable {
                Interactable::Button(true) => anim_target.to_transform,
                Interactable::Button(false) => anim_target.from_transform,
                Interactable::Lever(progress) => anim_target.from_transform,
            }
        }
    }
}

fn update_interaction_candidate(
    mut candidate: Single<&mut InteractionCandidate, With<Player>>,
    ray_caster: Single<(&RayCaster, &RayHits), With<PlayerCamera>>,
) {
    let (ray, hits) = ray_caster.into_inner();
    if let Some(hit) = hits.iter_sorted().next() {
        candidate.0 = Some(hit.entity);
        info!("Found interactable: {:?}", hit.entity);
    } else {
        candidate.0 = None;
    }
}
