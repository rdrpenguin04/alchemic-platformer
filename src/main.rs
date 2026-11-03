use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tiled::prelude::*;

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
        .add_plugins(TiledPlugin::default())
        .add_systems(Startup, startup)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .run()
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let map_handle = asset_server.load("levels/level1.tmx");

    commands.spawn(TiledMap(map_handle));
}
