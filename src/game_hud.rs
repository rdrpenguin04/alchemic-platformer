use std::fmt::Write as _;

use bevy::{image::TextureAtlasTemplate, prelude::*};

use crate::{
    Player,
    alchemy::{Element, Elements},
};

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

fn spawn_hud(mut commands: Commands, hud_assets: Res<HudAssets>, elements: Res<Elements>) {
    let base_elements = ["fire", "air", "water", "earth"];
    let element_nodes: Vec<Box<dyn SceneList>> = base_elements
        .iter().flat_map(|x| elements.by_id.get(*x))
        .map(|Element { id, name, icon_id }| {
            let id = id.clone();
            let name = name.clone();
            let icon_id = *icon_id;
            Box::from(bsn_list! {
                ElementText({ id.clone() })
                Text("00")
                --
                ImageNode {
                    image: { hud_assets.icons_image.clone() },
                    texture_atlas: { Some(TextureAtlasTemplate {
                        layout: { hud_assets.icons_atlas_layout.clone().into() },
                        index: icon_id,
                    }.into()) }
                }
                Node {
                    width: px(32),
                    height: px(32),
                }
                Pickable {
                    is_hoverable: true
                }
                on({
                    move |mut event: On<PointerDragStart>, mut commands: Commands, player: Single<&Player>, hud_assets: Res<HudAssets>, elements: Res<Elements>| {
                        event.propagate(false);
                        let Some(pos) = event.hit.position else {
                            return;
                        };
                        println!("{pos:?}");
                        if player.inventory.get(&id).is_some_and(|x| *x > 0) {
                            let element = elements.by_id.get(&id).unwrap();
                            commands.spawn_scene(bsn! {
                                Sprite {
                                    image: { hud_assets.icons_image.clone() },
                                    texture_atlas: { Some(TextureAtlasTemplate {
                                        layout: { hud_assets.icons_atlas_layout.clone().into() },
                                        index: icon_id,
                                    }.into()) }
                                }
                                Transform::from_translation(pos + vec3(0.0, 0.0, 10.0))
                            });
                        }
                    }
                })
                --
                Text({ name.clone() })
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
