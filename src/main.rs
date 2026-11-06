#![feature(char_is_xid)]

use std::fmt::Write as _;

use avian2d::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    dev_tools::fps_overlay::FpsOverlayPlugin,
    image::{ImageLoaderSettings, ImageSampler, TextureAtlasTemplate},
    platform::collections::HashMap,
    prelude::*,
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

use crate::{alchemy::Recipe, helpers::JsonLoader};

mod alchemy;
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
        .add_systems(Startup, startup)
        .add_systems(Update, simple_input.in_set(TnuaUserControlsSystems))
        .add_systems(Update, (passive_element_accumulation, earth_mining))
        .add_systems(Update, (update_camera, update_element_text))
        .init_asset_loader::<JsonLoader<Recipe>>()
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            int_grid_rendering: IntGridRendering::Invisible,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .init_resource::<GameAssets>()
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
struct Player {
    inventory: HashMap<String, u8>,
    element_max: u8,
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
        }
    }
}

#[derive(Component, Default, Clone, Deref)]
struct ElementText(String);

#[derive(Resource)]
struct GameAssets {
    icons_image: Handle<Image>,
    icons_atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let icons_image = asset_server.load("tileset/AlchemyIcons.png");

        let mut texture_atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();

        let icons_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(8),
            4,
            5,
            None,
            None,
        ));

        Self {
            icons_image,
            icons_atlas_layout,
        }
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, game_assets: Res<GameAssets>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Transform::from_scale(vec3(0.25, 0.25, 1.0))
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
        CollisionLayers::new(
            [LayerNames::Default, LayerNames::Player],
            [LayerNames::Default, LayerNames::Player],
        )
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
        // LockedAxes::ROTATION_LOCKED
        Transform::from_translation(vec3(32.0, 200.0, 3.0))
        GravityScale(10.0)
        TransformInterpolation
        CollidingEntities
    });

    let elements = [
        (4, "Fire", "fire"),
        (8, "Air", "air"),
        (12, "Water", "water"),
        (16, "Earth", "earth"),
    ];
    let element_nodes: Vec<Box<dyn SceneList>> = elements
        .iter()
        .map(|&(index, text, name)| {
            Box::from(bsn_list! {
                ElementText(name)
                Text("00")
                --
                ImageNode {
                    image: { game_assets.icons_image.clone() },
                    texture_atlas: { Some(TextureAtlasTemplate {
                        layout: { game_assets.icons_atlas_layout.clone().into() },
                        index,
                    }.into()) }
                }
                Node {
                    width: px(32),
                    height: px(32),
                }
                --
                Text(text)
            })
        })
        .collect();

    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            top: px(8),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
            ],
            column_gap: px(8),
            row_gap: px(5),
            align_items: AlignItems::Center,
        }
        Children [
            {element_nodes}
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
        controller.action(ControlScheme::Jump(default()));
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

fn update_element_text(element_texts: Query<(&ElementText, &mut Text)>, player: Single<&Player>) {
    for (name, mut text) in element_texts {
        if let Some(el) = player.inventory.get(&name.0) {
            text.0.clear();
            write!(text.0, "{el:02}").unwrap();
        }
    }
}

fn passive_element_accumulation(
    mut player: Single<(&mut Player, &CollidingEntities)>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    water_tiles: Query<Entity, With<Water>>,
) {
    let (player, collisions) = &mut *player;
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(1.0, TimerMode::Repeating));
    timer.tick(time.delta());

    let max = player.element_max;

    if timer.just_finished() {
        let air = player.inventory.get_mut("air").unwrap();
        *air = (*air + 1).clamp(0, max);

        let water = player.inventory.get_mut("water").unwrap();
        let mut water_add = 0;
        for entity in collisions.iter() {
            if water_tiles.contains(*entity) {
                water_add = 1;
            }
        }
        *water = (*water + water_add).clamp(0, max);
    }
}

fn earth_mining(
    mut player: Single<(&mut Player, &TnuaController<ControlScheme>)>,
    earth_tiles: Query<Entity, With<Earth>>, // TODO: detect "earth", not "solid collision". Probably needs a better int grid enumeration.
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    click: Res<ButtonInput<MouseButton>>,
) {
    let (player, controller) = &mut *player;
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.4, TimerMode::Repeating));

    timer.tick(time.delta());
    println!("{:?}", controller.basis_memory);

    if let Some(e) = controller.basis_memory.standing_on_entity()
        && earth_tiles.contains(e)
        && click.pressed(MouseButton::Left)
    {
        if timer.just_finished() {
            let max = player.element_max;
            let earth = player.inventory.get_mut("earth").unwrap();
            *earth = (*earth + 1).clamp(0, max);
        }
    } else {
        timer.reset();
    }
}
