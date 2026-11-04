use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tnua::{
    TnuaGhostPlatform,
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    control_helpers::TnuaSimpleFallThroughPlatformsHelper,
    prelude::*,
};
use bevy_tnua_avian2d::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Alchemic Platformer".into(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        )
        .add_plugins((
            TiledPlugin::default(),
            TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
            PhysicsPlugins::default().with_length_unit(100.0),
            PhysicsDebugPlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian2dPlugin::new(FixedUpdate),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, simple_input.in_set(TnuaUserControlsSystems))
        .add_systems(Update, move_camera)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .register_type::<Semisolid>()
        .run()
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Semisolid;

#[derive(Component, Default, Clone)]
struct Player;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Transform::from_scale(vec3(0.3, 0.3, 1.0))
    });

    let map_handle = asset_server.load("levels/level1-new.tmx");

    commands
        .spawn((
            TiledMap(map_handle),
        ))
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>,
             mut commands: Commands,
             query: Query<Has<Semisolid>>| {
                let mut entity = commands.entity(collider_created.event().origin);
                entity.insert(RigidBody::Static);
                println!("hi1");
                if query.contains(collider_created.event().origin) {
                    println!("hi");
                    // entity.insert(TnuaGhostPlatform);
                }
            },
        );

    commands.spawn_scene(bsn! {
        Player
        Mesh2d(asset_value(Capsule2d {
            radius: 3.0,
            half_length: 3.0
        }))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        RigidBody::Dynamic
        Collider::capsule(3.0, 6.0)
        TnuaController<ControlScheme>
        TnuaConfig<ControlScheme>(asset_value(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                speed: 128.0,
                float_height: 3.5,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 128.0,
                ..default()
            }
        }))
        TnuaAvian2dSensorShape(Collider::rectangle(6.0, 9.5))
        TnuaSimpleFallThroughPlatformsHelper
        LockedAxes::ROTATION_LOCKED
        Transform::from_translation(vec3(0.0, 200.0, 3.0))
        GravityScale(10.0)
        TransformInterpolation
    });
}

fn simple_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController<ControlScheme>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };
    controller.initiate_action_feeding();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    controller.basis = TnuaBuiltinWalk {
        desired_motion: direction.normalize_or_zero(),
        ..default()
    };

    if keyboard.pressed(KeyCode::Space) {
        controller.action(ControlScheme::Jump(default()));
    }
}

const CAMERA_DECAY_RATE: f32 = 2.0;

fn move_camera(player: Single<&Transform, With<Player>>, mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>, time: Res<Time>) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
