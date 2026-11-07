use std::fmt::Write as _;

use bevy::{image::TextureAtlasTemplate, prelude::*};

use crate::Player;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud)
        .add_systems(Update, update_element_text)
        .init_resource::<HudAssets>();
}

#[derive(Component, Default, Clone, Deref)]
struct ElementText(String);

#[derive(Resource)]
struct HudAssets {
    icons_image: Handle<Image>,
    icons_atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for HudAssets {
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

fn spawn_hud(mut commands: Commands, hud_assets: Res<HudAssets>) {
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
                    image: { hud_assets.icons_image.clone() },
                    texture_atlas: { Some(TextureAtlasTemplate {
                        layout: { hud_assets.icons_atlas_layout.clone().into() },
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

fn update_element_text(element_texts: Query<(&ElementText, &mut Text)>, player: Single<&Player>) {
    for (name, mut text) in element_texts {
        if let Some(el) = player.inventory.get(&name.0) {
            text.0.clear();
            write!(text.0, "{el:02}").unwrap();
        }
    }
}
