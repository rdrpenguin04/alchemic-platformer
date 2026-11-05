use avian2d::prelude::*;
use bevy::{
    asset::AssetMetaCheck, dev_tools::fps_overlay::FpsOverlayPlugin, image::{ImageLoaderSettings, ImageSampler}, prelude::*,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tnua::{
    TnuaGhostOverwrites, TnuaGhostPlatform, TnuaGhostSensor, TnuaSensorsEntities,
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
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins((
            LdtkPlugin,
            PhysicsPlugins::default().with_length_unit(8.0),
            // PhysicsDebugPlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian2dPlugin::new(FixedUpdate),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, simple_input.in_set(TnuaUserControlsSystems))
        .add_systems(Update, update_camera)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            int_grid_rendering: IntGridRendering::Invisible,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .register_ldtk_int_cell::<Wall>(1)
        .register_ldtk_int_cell::<Semisolid>(2)
        .register_ldtk_int_cell::<Water>(3)
        .run()
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

#[derive(PhysicsLayer, Default)]
enum LayerNames {
    #[default]
    Default,
    Player,
    FallThrough,
}

// Note: is a fieldless struct rather than a unit struct due to a ldtk proc-macro bug
#[derive(Component, Reflect, LdtkIntCell)]
#[require(RigidBody::Static, Collider::rectangle(8.0, 8.0))]
#[reflect(Component)]
struct Wall {}

// Note: is a fieldless struct rather than a unit struct due to a ldtk proc-macro bug
#[derive(Component, Reflect, LdtkIntCell)]
#[require(
    RigidBody::Static,
    Collider::rectangle(8.0, 8.0),
    TnuaGhostPlatform,
    CollisionLayers::new(
        [LayerNames::FallThrough],
        [LayerNames::FallThrough]
    ),
)]
#[reflect(Component)]
struct Semisolid {}

// Note: is a fieldless struct rather than a unit struct due to a ldtk proc-macro bug
#[derive(Component, Reflect, LdtkIntCell)]
#[reflect(Component)]
struct Water {}

#[derive(Component, Default, Clone)]
struct Player;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Transform::from_scale(vec3(0.3, 0.3, 1.0))
        Children [
            Sprite {
                image: {
                    asset_server
                        .load_builder()
                        .with_settings(|s: &mut _|
                            *s = ImageLoaderSettings {
                                sampler: ImageSampler::linear(),
                                ..default()
                            }
                        ).load("tileset/bg.png")
                },
            }
            Transform::from_translation(vec3(0.0, 0.0, -3.0)).with_scale(vec3(200.0, 100.0, 1.0))
        ]
    });

    commands.spawn_scene(bsn! {
        LdtkProjectHandle { handle: "ldtk-project.ldtk" }
    });

    // commands.spawn(LdtkWorldBundle {
    //     ldtk_handle: asset_server.load("ldtk-project.ldtk").into(),
    //     ..default()
    // });

    commands.spawn_scene(bsn! {
        Player
        Mesh2d(asset_value(Capsule2d {
            radius: 3.0,
            half_length: 3.5
        }))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        RigidBody::Dynamic
        Collider::capsule(3.0, 4.0)
        TnuaAvian2dSensorShape(Collider::rectangle(5.9, 0.0))
        TnuaController<ControlScheme>
        TnuaConfig<ControlScheme>(asset_value(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                speed: 64.0,
                float_height: 7.0,
                acceleration: 192.0,
                air_acceleration: 64.0,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 80.0,
                takeoff_extra_gravity: 80.0,
                fall_extra_gravity: 64.0,
                shorten_extra_gravity: 300.0,
                ..default()
            }
        }))
        TnuaGhostOverwrites<ControlScheme>
        TnuaSimpleFallThroughPlatformsHelper
        LockedAxes::ROTATION_LOCKED
        Transform::from_translation(vec3(0.0, 200.0, 3.0))
        GravityScale(10.0)
        TransformInterpolation
    });
}

fn simple_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut TnuaController<ControlScheme>,
        &TnuaSensorsEntities<ControlScheme>,
        &mut TnuaGhostOverwrites<ControlScheme>,
        &mut TnuaSimpleFallThroughPlatformsHelper,
    )>,
    ghost_sensors_query: Query<&TnuaGhostSensor>,
) {
    let Ok((mut controller, sensors_entities, mut ghost_overwrites, mut fall_through_helper)) =
        query.single_mut()
    else {
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

    // one-way platforms
    if let Some(ghost_sensor) = sensors_entities
        .ground
        .and_then(|entity| ghost_sensors_query.get(entity).ok())
    {
        let mut handler = fall_through_helper.with(&mut ghost_overwrites.ground, ghost_sensor, 4.0);
        
        if keyboard.pressed(KeyCode::ArrowDown) {
            handler.try_falling(keyboard.just_pressed(KeyCode::ArrowDown));
        } else {
            handler.dont_fall();
        }
    }
}

const CAMERA_DECAY_RATE: f32 = 2.0;

fn update_camera(
    player: Single<&Transform, With<Player>>,
    mut camera: Single<(&mut Transform), (With<Camera2d>, Without<Player>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
