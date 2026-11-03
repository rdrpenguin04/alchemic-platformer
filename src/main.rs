use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;
use bevy_tnua::{
    TnuaGhostSensor,
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
        ))
        .add_systems(Startup, startup)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .run()
}

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum ControlScheme {
    Jump(TnuaBuiltinJump),
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let map_handle = asset_server.load("levels/level1.tmx");

    commands.spawn(TiledMap(map_handle));

    commands.spawn_scene(bsn! {
        Mesh2d(asset_value(Capsule2d {
            radius: 10.0,
            half_length: 11.0
        }))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        RigidBody::Dynamic
        Collider::capsule(10.0, 20.0)
        TnuaController<ControlScheme>
        TnuaConfig<ControlScheme>(asset_value(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                float_height: 20.0,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 4.0,
                ..default()
            }
        }))
        // TnuaAvian2dSensorShape(Collider::rectangle(0.95, 0.0))
        TnuaSimpleFallThroughPlatformsHelper
        LockedAxes::ROTATION_LOCKED
        Transform::from_translation(vec3(0.0, 400.0, 3.0))
        GravityScale(10.0)
    });
}
