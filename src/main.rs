#![feature(char_is_xid)]

use avian2d::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    image::{ImageLoaderSettings, ImageSampler},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_tnua::{
    TnuaGhostOverwrites, TnuaGhostPlatform, TnuaGhostSensor, TnuaSensorsEntities,
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    control_helpers::TnuaSimpleFallThroughPlatformsHelper,
    prelude::*,
};
use bevy_tnua_avian2d::prelude::*;

mod alchemy;
mod game_hud;
mod helpers;

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
        // .add_plugins(FpsOverlayPlugin::default())
        .add_plugins((
            LdtkPlugin,
            PhysicsPlugins::default().with_length_unit(8.0),
            // PhysicsDebugPlugin,
            TnuaControllerPlugin::<ControlScheme>::new(FixedUpdate),
            TnuaAvian2dPlugin::new(FixedUpdate),
            // EguiPlugin::default(),
            // WorldInspectorPlugin::default(),
        ))
        .add_plugins((alchemy::plugin, game_hud::plugin))
        .add_systems(Startup, startup)
        .add_systems(Update, simple_input.in_set(TnuaUserControlsSystems))
        .add_systems(PreUpdate, check_in_water)
        .add_systems(Update, (passive_element_accumulation, earth_mining))
        .add_systems(
            Update,
            (
                update_camera,
                update_player_eye,
                update_player_physics,
            ),
        )
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            int_grid_rendering: IntGridRendering::Invisible,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .register_ldtk_int_cell::<EarthWall>(1)
        .register_ldtk_int_cell::<EarthFloor>(2)
        .register_ldtk_int_cell::<Water>(3)
        .register_ldtk_int_cell::<NonEarthWall>(4)
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

// Int cells
// Note: There are fieldless structs rather than unit structs due to a ldtk proc-macro bug

#[derive(Component, Reflect, LdtkIntCell)]
#[require(Earth, Wall)]
struct EarthWall {}

#[derive(Component, Reflect, LdtkIntCell)]
#[require(Earth, Semisolid)]
struct EarthFloor {}

// Note: is a fieldless struct rather than a unit struct due to a ldtk proc-macro bug
#[derive(Component, Reflect, LdtkIntCell)]
#[require(Sensor, Collider::rectangle(8.0, 8.0))]
#[reflect(Component)]
struct Water {}

#[derive(Component, Reflect, LdtkIntCell)]
#[require(Wall)]
struct NonEarthWall {}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Earth;

#[derive(Component, Default, Reflect)]
#[require(RigidBody::Static, Collider::rectangle(8.0, 8.0))]
#[reflect(Component)]
struct Wall;

#[derive(Component, Default, Reflect)]
#[component(storage = "SparseSet")]
struct InWater;

// Note: is a fieldless struct rather than a unit struct due to a ldtk proc-macro bug
#[derive(Component, Default, Reflect)]
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
struct Semisolid;

#[derive(Component, Clone)]
pub struct Player {
    pub inventory: HashMap<String, u8>,
    pub element_max: u8,
    pub health: u8,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            inventory: [
                ("fire".into(), 0),
                ("air".into(), 0),
                ("water".into(), 0),
                ("earth".into(), 0),
            ]
            .into(),
            element_max: 10,
            health: 10,
        }
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Transform::from_translation(vec3(160.0, 200.0, 0.0)).with_scale(vec3(0.25, 0.25, 1.0))
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
        #Player
        Player
        Mesh2d(asset_value(Capsule2d {
            radius: 3.0,
            half_length: 3.5
        }))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        RigidBody::Dynamic
        Collider::capsule(3.0, 4.0)
        CollisionLayers::new(
            [LayerNames::Default, LayerNames::Player],
            [LayerNames::Default, LayerNames::Player],
        )
        TnuaAvian2dSensorShape(Collider::rectangle(5.98, 0.0))
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
        // LockedAxes::ROTATION_LOCKED
        Transform::from_translation(vec3(160.0, 200.0, 3.0))
        GravityScale(12.0)
        TransformInterpolation
        CollidingEntities
        Children [
            #Eye
            Mesh2d(asset_value(Circle { radius: 0.5 }))
            MeshMaterial2d<ColorMaterial>(asset_value(Color::BLACK))
            Transform::from_translation(vec3(2.0, 2.0, 1.0))
        ]
    });
}

fn simple_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut TnuaController<ControlScheme>,
        &TnuaSensorsEntities<ControlScheme>,
        &mut TnuaGhostOverwrites<ControlScheme>,
        &mut TnuaSimpleFallThroughPlatformsHelper,
        Has<InWater>,
    )>,
    ghost_sensors_query: Query<&TnuaGhostSensor>,
) {
    let Ok((
        mut controller,
        sensors_entities,
        mut ghost_overwrites,
        mut fall_through_helper,
        in_water,
    )) = query.single_mut()
    else {
        return;
    };
    controller.initiate_action_feeding();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    controller.basis = TnuaBuiltinWalk {
        desired_motion: direction.normalize_or_zero(),
        ..default()
    };

    if keyboard.pressed(KeyCode::ArrowUp)
        || keyboard.pressed(KeyCode::KeyW)
        || keyboard.pressed(KeyCode::Space)
    {
        controller.action(ControlScheme::Jump(TnuaBuiltinJump {
            allow_in_air: in_water,
            ..default()
        }));
    }

    // one-way platforms
    if let Some(ghost_sensor) = sensors_entities
        .ground
        .and_then(|entity| ghost_sensors_query.get(entity).ok())
    {
        let mut handler = fall_through_helper.with(&mut ghost_overwrites.ground, ghost_sensor, 4.0);

        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            handler.try_falling(
                keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS),
            );
        } else {
            handler.dont_fall();
        }
    }
}

const CAMERA_DECAY_RATE: f32 = 2.0;

fn update_camera(
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
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

fn passive_element_accumulation(
    mut player: Single<(&mut Player, Has<InWater>)>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let (player, in_water) = &mut *player;
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(1.0, TimerMode::Repeating));
    timer.tick(time.delta());

    let max = player.element_max;

    if timer.just_finished() {
        let air = player.inventory.get_mut("air").unwrap();
        *air = (*air + 1).clamp(0, max);

        let water = player.inventory.get_mut("water").unwrap();
        if *in_water {
            *water = (*water + 1).clamp(0, max);
        }
    }
}

fn earth_mining(
    mut player: Single<(&mut Player, &TnuaController<ControlScheme>)>,
    earth_tiles: Query<Entity, With<Earth>>,
    mut timer: Local<Option<Timer>>,
    mut counter: Local<u8>,
    time: Res<Time>,
    click: Res<ButtonInput<MouseButton>>,
) {
    let (player, controller) = &mut *player;
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.2, TimerMode::Repeating));

    timer.tick(time.delta());

    if let Some(e) = controller.basis_memory.standing_on_entity()
        && earth_tiles.contains(e)
        && click.pressed(MouseButton::Left)
    {
        if timer.just_finished() {
            *counter += 1;
            // TODO: particle effect, sound effect
            if *counter == 3 {
                *counter = 0;
                let max = player.element_max;
                let earth = player.inventory.get_mut("earth").unwrap();
                *earth = (*earth + 1).clamp(0, max);
            }
        }
    } else {
        timer.reset();
        *counter = 0;
    }
}

fn update_player_physics(
    mut player: Single<(&mut GravityScale, &TnuaConfig<ControlScheme>, Has<InWater>), With<Player>>,
    mut was_in_water: Local<bool>,
    mut config_assets: ResMut<Assets<ControlSchemeConfig>>,
) {
    let (gravity_scale, controller_config, in_water) = &mut *player;
    let mut controller_config = config_assets.get_mut(&controller_config.0).unwrap();
    if *in_water && !*was_in_water {
        gravity_scale.0 = 1.0;
        controller_config.basis.acceleration = 20.0;
        controller_config.basis.air_acceleration = 32.0;
        controller_config.jump.height = 20.0;
        controller_config.jump.shorten_extra_gravity = 0.0;
    } else if !*in_water && *was_in_water {
        gravity_scale.0 = 12.0;
        controller_config.basis.acceleration = 192.0;
        controller_config.basis.air_acceleration = 64.0;
        controller_config.jump.height = 80.0;
        controller_config.jump.shorten_extra_gravity = 300.0;
    }
    *was_in_water = *in_water;
}

fn check_in_water(
    mut commands: Commands,
    player: Single<(Entity, &CollidingEntities)>,
    water_tiles: Query<Entity, With<Water>>,
) {
    let (player, collisions) = *player;
    let mut in_water = false;
    for entity in collisions.iter() {
        if water_tiles.contains(*entity) {
            in_water = true;
        }
    }

    let mut player = commands.entity(player);
    if in_water {
        player.insert(InWater);
    } else {
        player.remove::<InWater>();
    }
}

// Exists to give a sense of "forward"
fn update_player_eye(
    player: Single<(&TnuaController<ControlScheme>, &LinearVelocity, &Children), With<Player>>,
    mut transform: Query<&mut Transform>,
    mut prev_target_x: Local<f32>,
    time: Res<Time>,
) {
    let (controller, vel, children) = *player;
    let mut eye_transform = transform.get_mut(children[0]).unwrap();

    let want_x = controller.basis.desired_motion.x;
    let want_x = if want_x == 0.0 {
        prev_target_x.signum()
    } else {
        want_x.signum()
    };
    *prev_target_x = want_x;

    let target = vec2(
        want_x * 2.0 + (vel.x / 30.0).atan() * 0.2,
        2.0 + (vel.y / 10.0).atan() * 1.0,
    );
    let mut cur = eye_transform.translation.xy();
    cur.smooth_nudge(&target, 24.0, time.delta_secs());

    eye_transform.translation.x = cur.x;
    eye_transform.translation.y = cur.y;
}
