use avian3d::prelude::*;
use bevy::{prelude::*, window::CursorGrabMode};
use bevy_enhanced_input::prelude::*;
use bevy_tnua::{TnuaUserControlsSystemSet, prelude::*};
use bevy_tnua_avian3d::{TnuaAvian3dPlugin, TnuaAvian3dSensorShape};

use crate::{menus::Menu, screens::Screen};

const CAPSULE_RADIUS: f32 = 0.5;
const CAPSULE_LENGTH: f32 = 1.0;
const PLAYER_HEIGHT: f32 = CAPSULE_LENGTH + 2.0 * CAPSULE_RADIUS;
const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.0;
const FLOAT_HEIGHT: f32 = PLAYER_HALF_HEIGHT + 0.01;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_plugins(TnuaControllerPlugin::new(FixedUpdate))
        .add_plugins(TnuaAvian3dPlugin::new(FixedUpdate))
        .add_input_context::<FirstPerson>()
        .add_systems(
            OnEnter(Menu::None),
            (capture_cursor, insert_player_actions).run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            OnExit(Menu::None),
            (release_cursor, remove_player_actions).run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(FixedLast, clear_movement)
        .add_systems(
            FixedUpdate,
            apply_movement.in_set(TnuaUserControlsSystemSet),
        )
        .add_observer(setup_player)
        .add_observer(action_binding)
        .add_observer(handle_rotation)
        .add_observer(handle_horizontal_movement);
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
#[require(Walking)]
pub struct Player;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerCamera;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component, Default)]
struct Walking(Vec2);

#[derive(InputContext)]
struct FirstPerson;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Rotate;

fn setup_player(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands.entity(trigger.target()).insert((
        StateScoped(Screen::Gameplay),
        RigidBody::Dynamic,
        TnuaController::default(),
        TnuaAvian3dSensorShape(Collider::cylinder(CAPSULE_RADIUS - 0.01, 0.0)),
        Collider::capsule(CAPSULE_RADIUS, CAPSULE_LENGTH),
        LockedAxes::ROTATION_LOCKED,
        // Movement feels nicer without friction.
        Friction {
            dynamic_coefficient: 0.0,
            static_coefficient: 0.0,
            combine_rule: CoefficientCombine::Multiply,
        },
    ));
}

/*
fn setup_camera(trigger: Trigger<OnAdd, PlayerCamera>, mut commands: Commands) {
    let query_filter = SpatialQueryFilter::from_mask(GameLayer::Interactable);
    commands.entity(trigger.target()).insert(
        RayCaster::default()
            .with_max_hits(1)
            .with_max_distance(15.0)
            .with_query_filter(query_filter),
    );
}
*/

// To define bindings for actions, write an observer for `Binding`.
// It's also possible to create bindings before the insertion,
// but this way you can conveniently reload bindings when settings change.
fn action_binding(
    trigger: Trigger<Binding<FirstPerson>>,
    mut players: Query<&mut Actions<FirstPerson>>,
) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    // Bindings like WASD or sticks are very common,
    // so we provide built-ins to assign all keys/axes at once.
    // We don't assign any conditions and in this case the action will
    // be triggered with any non-zero value.
    // An action can have multiple inputs bound to it
    // and will respond to any of them.
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            //SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            DeltaScale,
            Scale::splat(100.0),
        ));

    actions.bind::<Rotate>().to((
        // You can attach modifiers to individual inputs as well.
        Input::mouse_motion().with_modifiers((Scale::splat(0.1), Negate::all())),
        Axial::right_stick().with_modifiers_each((Scale::splat(2.0), Negate::x())),
    ));
}

fn apply_movement(player: Single<(&mut TnuaController, &Walking)>) {
    let (mut controller, walking) = player.into_inner();

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: Vec3::new(walking.0.x, 0.0, walking.0.y) * 7.,
        float_height: FLOAT_HEIGHT,
        ..default()
    });
}

fn handle_horizontal_movement(
    trigger: Trigger<Fired<Move>>,
    camera: Single<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player: Single<&mut Walking, (With<Player>, Without<Camera3d>)>,
) {
    let forward = camera.forward().xz().normalize_or_zero();
    let right = camera.right().xz().normalize_or_zero();

    //let direction = (right * trigger.value.x + forward * trigger.value.y).normalize_or_zero();
    player.0 += 0.5 * right * trigger.value.x + forward * trigger.value.y;
}

fn clear_movement(mut walkers: Query<&mut Walking>) {
    for mut walking in &mut walkers {
        walking.0 = Vec2::ZERO;
    }
}

fn handle_rotation(
    trigger: Trigger<Fired<Rotate>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let (mut yaw, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);

    yaw += trigger.value.x.to_radians();
    pitch += trigger.value.y.to_radians();

    pitch = pitch.clamp(-1.54, 1.54);

    // Order is important to prevent unintended roll
    camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

fn capture_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

fn release_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

fn insert_player_actions(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands
        .entity(player.into_inner())
        .insert(Actions::<FirstPerson>::default());
}

fn remove_player_actions(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands
        .entity(player.into_inner())
        .remove::<Actions<FirstPerson>>();
}
