use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::{events::Completed, prelude::InputAction};

use crate::{gameplay::event_handling, reactorview};

use super::{
    GameLayer,
    //physics::GameLayer,
    player::{Player, PlayerCamera},
};

pub fn plugin(app: &mut App) {
    app.register_type::<Interactable>()
        .register_type::<InteractableEventSource>()
        .register_type::<InteractableAnimationTarget>()
        .add_systems(Update, update_interactable_animation_targets)
        .add_systems(Update, update_interaction_candidate)
        .add_observer(on_add_interactable_animation_target)
        .add_observer(on_add_interactable)
        .add_observer(handle_interaction);
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

fn on_add_interactable(
    trigger: Trigger<OnAdd, Interactable>,
    mut commands: Commands,
    children: Query<&Children>,
    //colliders: Query<Entity, Or<(With<Collider>, With<ColliderConstructor>)>>,
) {
    commands
        .entity(trigger.target())
        .insert((RigidBody::Static, Sensor));

    children
        .iter_descendants(trigger.target())
        //.flat_map(|child| colliders.get(child))
        .for_each(|child| {
            commands.entity(child).insert(CollisionLayers::new(
                GameLayer::Interactable,
                LayerMask::ALL,
            ));
        });
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Interact;

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component, Default)]
pub enum InteractableEventSource {
    #[default]
    None,
    CycleDisplayMode {
        display: usize,
    },
    MoveControlRod {
        cell: usize,
        delta: f32,
    },
    SelectControlRod {
        cell: usize,
    },
    MoveSelectedControlRod {
        delta: f32,
    },
    ToggleValve {
        valve: usize,
    },
    ChangePumpPower {
        pump: usize,
        delta: f32,
    },
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
    player: Single<&GlobalTransform, With<PlayerCamera>>,
    spatial_query: SpatialQuery,
    parents: Query<&ChildOf>,
    interactables: Query<Entity, With<Interactable>>,
) {
    let transform = player.compute_transform();
    let hit = spatial_query.cast_ray(
        transform.translation,
        transform.forward(),
        3.5,
        true,
        &SpatialQueryFilter::from_mask(GameLayer::Interactable),
    );
    if let Some(hit) = hit {
        candidate.0 = parents
            .iter_ancestors(hit.entity)
            .find(|parent| interactables.get(*parent).is_ok());
    } else {
        candidate.0 = None;
    }
}

fn handle_interaction(
    _trigger: Trigger<Completed<Interact>>,
    mut commands: Commands,
    interaction_candidate: Single<&InteractionCandidate, With<Player>>,
    event_sources: Query<&InteractableEventSource>,
) {
    info!("Interaction triggered");
    if let Some(entity) = interaction_candidate.0 {
        info!("We have an interaction candidate: {}", entity);
        match event_sources.get(entity) {
            Ok(InteractableEventSource::CycleDisplayMode { display }) => {
                info!("Cycling display mode");
                commands.trigger(reactorview::view::CycleDisplayMode(*display));
            }
            Ok(InteractableEventSource::SelectControlRod { cell }) => {
                info!("Switching selected control rod to cell {cell}");
                commands.trigger(event_handling::SelectControlRod(*cell));
            }
            Ok(InteractableEventSource::MoveSelectedControlRod { delta }) => {
                info!("Moving selected control rod by {delta}");
                commands.trigger(event_handling::MoveSelectedControlRod(*delta));
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
