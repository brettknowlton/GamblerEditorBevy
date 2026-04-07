use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui::{self, RichText}};

use super::*;

macro_rules! send_message {
    ($key:expr, $messages:expr, $message:expr) => {
        $messages.messages.push(($key, $message.to_string()));
    };
    (_) => {
        $messages.messages.push((None, " ".to_string()));
    };
}

#[derive(Component)]
pub struct DisplayMessage;

#[derive(Clone, Debug, Default)]
pub struct ToolingMenuItem {
    pub id: u64,
    pub label: String,
    pub texture_key: Option<char>,
    pub rect: Option<Rect>,
}

#[derive(Resource, Clone, Debug)]
pub struct ToolingMenuState {
    pub title: String,
    pub items: Vec<ToolingMenuItem>,
    pub selected_item_id: Option<u64>,
    pub visible: bool,
}

impl Default for ToolingMenuState {
    fn default() -> Self {
        Self {
            title: String::new(),
            items: vec![],
            selected_item_id: None,
            visible: false,
        }
    }
}

#[derive(Component)]
pub struct ToolingMenuRoot;

#[derive(Component)]
pub struct ToolingMenuTitle;

#[derive(Component)]
pub struct ToolingMenuContent;

#[derive(Component)]
pub struct ToolingMenuItemNode {
    pub id: u64,
}

pub fn left_panel(
    contexts: &mut EguiContexts,
    editor_state: &Res<State<EditorState>>,
    input: &Res<ButtonInput<KeyCode>>,
    tooling_menu_state: &mut ResMut<ToolingMenuState>,
    textures: &Res<TextureHandles>,
    selected_tile_id: &mut ResMut<SelectedTileID>,
    placeholder_update_writer: &mut MessageWriter<UpdatePlaceholderMessage>,
) -> Result {
    if !tooling_menu_state.visible || !matches!(editor_state.get(), EditorState::Editing(_)) {
        return Ok(());
    }

    let items = tooling_menu_state.items.clone();
    if items.is_empty() {
        return Ok(());
    }

    let mut current_index = items
        .iter()
        .position(|item| Some(item.id) == tooling_menu_state.selected_item_id)
        .unwrap_or(0);

    let is_tile_mode = matches!(
        editor_state.get(),
        EditorState::Editing(EditingComponent::Tile)
    );

    let columns = if is_tile_mode {
        SPRITESHEET_WIDTH as usize / 2
    } else {
        1
    }
    .max(1);

    let tile_texture_id = if is_tile_mode {
        textures
            .0
            .get(&'t')
            .map(|handle| contexts.add_image(EguiTextureHandle::Strong(handle.clone())))
    } else {
        None
    };

    let ctx = contexts.ctx_mut()?;

    if input.just_pressed(KeyCode::ArrowRight) {
        current_index = (current_index + 1) % items.len();
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
        current_index = if current_index == 0 {
            items.len() - 1
        } else {
            current_index - 1
        };
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        current_index = (current_index + columns) % items.len();
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        current_index = if current_index >= columns {
            current_index - columns
        } else {
            (items.len() + current_index - columns % items.len()) % items.len()
        };
    }

    let mut next_selected_id = Some(items[current_index].id);
    let tile_button_px = TILE_SIZE as f32;
    let tile_spacing = 4.0;
    let panel_width = if is_tile_mode {
        // Width = N buttons + spacing + padding for scroll bar/margins.
        (columns as f32 * tile_button_px)
            + ((columns.saturating_sub(1)) as f32 * tile_spacing)
            + 24.0
    } else {
        220.0
    };

    egui::SidePanel::left("tooling_menu_panel")
        .frame(egui::Frame {
            fill: egui::Color32::from_rgba_unmultiplied(18, 22, 30, 165),
            stroke: egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(220, 230, 245, 120),
            ),
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        })
        .resizable(false)
        .default_width(panel_width)
        .show(ctx, |ui| {
            ui.heading(RichText::new(tooling_menu_state.title.clone()).strong().size(18.0).color(egui::Color32::from_rgba_unmultiplied(220, 230, 245, 255)));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if is_tile_mode {
                    egui::Grid::new("tooling_grid")
                        .num_columns(columns)
                        .spacing([tile_spacing, tile_spacing])
                        .show(ui, |ui| {
                            for (i, item) in items.iter().enumerate() {
                                let selected = Some(item.id) == next_selected_id;
                                let tile_button_size = egui::vec2(tile_button_px, tile_button_px);
                                let (rect, response) =
                                    ui.allocate_exact_size(tile_button_size, egui::Sense::click());

                                let bg = if selected {
                                    egui::Color32::from_rgba_unmultiplied(245, 230, 120, 40)
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
                                };
                                ui.painter().rect_filled(rect, 4.0, bg);

                                if let (Some(texture_id), Some(tile_rect)) =
                                    (tile_texture_id, item.rect)
                                {
                                    let atlas_w = (SPRITESHEET_WIDTH * TILE_SIZE as u64) as f32;
                                    let rows = ((MAX_SPRITESHEET_ITEMS - SPRITESHEET_WIDTH)
                                        / SPRITESHEET_WIDTH)
                                        as f32;
                                    let atlas_h = (rows * TILE_SIZE as f32).max(TILE_SIZE as f32);
                                    let uv = egui::Rect::from_min_max(
                                        egui::pos2(
                                            tile_rect.min.x / atlas_w,
                                            tile_rect.min.y / atlas_h,
                                        ),
                                        egui::pos2(
                                            tile_rect.max.x / atlas_w,
                                            tile_rect.max.y / atlas_h,
                                        ),
                                    );

                                    let image_rect = rect.shrink2(egui::vec2(2.0, 2.0));
                                    ui.painter().image(
                                        texture_id,
                                        image_rect,
                                        uv,
                                        egui::Color32::WHITE,
                                    );
                                }

                                let stroke_color = if selected {
                                    egui::Color32::from_rgb(245, 230, 120)
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(210, 220, 235, 120)
                                };
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(1.0, stroke_color),
                                    egui::StrokeKind::Outside,
                                );

                                let label = item.label.as_str();
                                let text_pos = egui::pos2(rect.right() - 3.0, rect.bottom() - 2.0);
                                ui.painter().text(
                                    text_pos,
                                    egui::Align2::RIGHT_BOTTOM,
                                    label,
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(250, 250, 250),
                                );

                                if response.clicked() {
                                    next_selected_id = Some(item.id);
                                }

                                if (i + 1) % columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                } else {
                    for item in &items {
                        let selected = Some(item.id) == next_selected_id;
                        if ui.selectable_label(selected, item.label.as_str()).clicked() {
                            next_selected_id = Some(item.id);
                        }
                    }
                }
            });
        });

    if tooling_menu_state.selected_item_id != next_selected_id {
        tooling_menu_state.selected_item_id = next_selected_id;

        if is_tile_mode {
            if let Some(id) = next_selected_id {
                selected_tile_id.id = id;
                let rect = items
                    .iter()
                    .find(|item| item.id == id)
                    .and_then(|item| item.rect)
                    .unwrap_or(Rect {
                        min: Vec2::new(
                            (id % SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
                            (id / SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
                        ),
                        max: Vec2::new(
                            (id % SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
                            (id / SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
                        ),
                    });

                placeholder_update_writer.write(UpdatePlaceholderMessage {
                    tcoord: TCoordinate::new('t', Coordinate(0, 0)),
                    rect,
                });
            }
        }
    }

    Ok(())
}

pub fn egui_panel_render(
    mut contexts: EguiContexts,
    editor_state: Res<State<EditorState>>,
    input: Res<ButtonInput<KeyCode>>,
    mut tooling_menu_state: ResMut<ToolingMenuState>,
    textures: Res<TextureHandles>,
    mut selected_tile_id: ResMut<SelectedTileID>,
    mut placeholder_update_writer: MessageWriter<UpdatePlaceholderMessage>,
) -> Result {
    left_panel(
        &mut contexts,
        &editor_state,
        &input,
        &mut tooling_menu_state,
        &textures,
        &mut selected_tile_id,
        &mut placeholder_update_writer,
    )?;

    Ok(())
}

pub fn send_messages(
    mut queued_messages: ResMut<EditorBottomBarQueuedMessages>,
    mut display_message: ResMut<EditorBottomBarDisplayed>,
) {
    if let Some((_, message)) = queued_messages.messages.first() {
        display_message.text = format!("{message}",);
    }
    //push any messages into the in-game console and leave the last one in our BottomBarMessage for display
    let item = queued_messages.messages.pop();
    {
        match item {
            Some((k, m)) => {
                let k = k.unwrap_or('i');
                println!("{}:> {}", k, m);
            }
            None => {}
        }
    }
}

///Systems have been added for this component to keep all UI items moving at the same speed, and therefore always relatively positioned to eachother.
/// Useful for menus, or any thing that you want to keep moving based on the camera's location. This does not prevent movement of the object by other systems,
/// we are just also using this to TAG all UI items so we can easily find them in queries (typically for movement so far)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform)]
pub struct UIItem {
    pub vel_x: f32,
    pub vel_y: f32,
}

pub fn general_editor_ui(
    mut contexts: EguiContexts,
    editor_state: Res<State<EditorState>>,
    display_message: ResMut<EditorBottomBarDisplayed>,
) -> Result {
    let is_in_editor = *editor_state.get() != EditorState::Inactive;
    let ctx = contexts.ctx_mut()?;

    let panel_height = 30.0;

    let message_string = &display_message.text;

    egui::TopBottomPanel::bottom("bottom_panel")
        .frame(egui::Frame {
            fill: egui::Color32::from_rgba_unmultiplied(18, 22, 30, 165),
            stroke: egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(220, 230, 245, 120),
            ),
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        })
        .resizable(false)
        .default_height(panel_height)
        .show(ctx, |ui| {
            ui.add(egui::Label::new(RichText::new(message_string).strong().size(18.0).color(egui::Color32::from_rgba_unmultiplied(220, 230, 245, 255))));
        });

    Ok(())
}

pub fn sync_tooling_menu_visibility(
    editor_state: Res<State<EditorState>>,
    mut tooling_menu: ResMut<ToolingMenuState>,
) {
    if !editor_state.is_changed() {
        return;
    }

    if !matches!(editor_state.get(), EditorState::Editing(_)) {
        tooling_menu.visible = false;
    }
}

pub fn render_tooling_menu(
    mut commands: Commands,
    tooling_menu: Res<ToolingMenuState>,
    textures: Res<TextureHandles>,
    mut root_query: Query<&mut Visibility, With<ToolingMenuRoot>>,
    mut title_query: Query<&mut Text, With<ToolingMenuTitle>>,
    content_query: Query<Entity, With<ToolingMenuContent>>,
) {
    if !tooling_menu.is_changed() && !textures.is_changed() {
        return;
    }

    if let Ok(mut root_visibility) = root_query.single_mut() {
        *root_visibility = if tooling_menu.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut title) = title_query.single_mut() {
        **title = tooling_menu.title.clone();
    }

    if let Ok(content) = content_query.single() {
        if !tooling_menu.visible {
            commands.entity(content).despawn();
        }
        commands.entity(content).with_children(|parent| {
            for item in &tooling_menu.items {
                let is_selected = tooling_menu.selected_item_id == Some(item.id);
                let border_color = if is_selected {
                    Color::srgba(0.95, 0.9, 0.3, 1.0)
                } else {
                    Color::srgba(0.8, 0.84, 0.9, 0.3)
                };

                parent
                    .spawn((
                        ToolingMenuItemNode { id: item.id },
                        Node {
                            width: Val::Px(72.0),
                            min_height: Val::Px(72.0),
                            padding: UiRect::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.16, 0.22, 0.95)),
                        BorderColor::all(border_color),
                    ))
                    .with_children(|item_parent| {
                        if let Some(texture_key) = item.texture_key {
                            if let Some(handle) = textures.0.get(&texture_key) {
                                item_parent.spawn((
                                    Node {
                                        width: Val::Px(SCALED_TILE_WIDTH as f32),
                                        height: Val::Px(SCALED_TILE_HEIGHT as f32),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: handle.clone(),
                                        rect: item.rect,
                                        ..default()
                                    },
                                ));
                            }
                        }

                        item_parent.spawn((Text {
                            0: item.label.clone(),
                            ..default()
                        },));
                    });
            }
        });
    }
}

pub fn hide_tooling_menu(mut tooling_menu: ResMut<ToolingMenuState>) {
    tooling_menu.visible = false;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// A component that marks an entity as a placeholder object, these are preview objects that are not yet placed into the scene.
/// note: this is separate from our placeholder resources, we could create many of these if we are prepping to place a lot of items in one keypress
pub struct PlaceholderObjectTag;

pub fn update_placeholder<T: SignificantComponent + Component + Default>(
    mut commands: Commands,

    state: ResMut<State<EditorState>>,
    mut placeholder: ResMut<PlaceholderHandle>,
    textures: Res<TextureHandles>,

    crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    //delete any existing placeholder objects
    for (e, _) in placeholders.iter() {
        commands.entity(e).despawn();
    }

    let m = match state.get() {
        EditorState::Editing(EditingComponent::Tile) => 't',
        EditorState::Editing(EditingComponent::Collider) => 'c',
        _ => {
            'r' //use selection rects as fallback
        }
    };
    //update the placeholder object to be the major type of the current editing mode
    placeholder.0 = textures.0.get(&m).unwrap().clone();

    let Ok((_, t)) = crosshairs.single() else {
        return;
    };

    commands.spawn((
        T::default(),
        Sprite {
            image: placeholder.0.clone(),
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(t.translation.x, t.translation.y, UI_Z_LAYER),
            ..default()
        },
        UIItem { ..default() },
        ui::NormalModeUI, // give it normalModeUI so it will just be destroyed when we exit normalMode
        PlaceholderObjectTag, //tag it as a placeholder object so we can delete it when we switch from this mode.
    ));
}

pub fn trigger_placeholder_update(
    mut ev: MessageReader<UpdatePlaceholderMessage>,
    mut commands: Commands,

    state: ResMut<State<EditorState>>,
    placeholder: ResMut<PlaceholderHandle>,

    // crosshairs: Query<(&Crosshair, &Transform)>,
    placeholders: Query<(Entity, &PlaceholderObjectTag)>,
) {
    for e in ev.read() {
        println!("Placeholder Update Event Triggered");
        match state.get() {
            EditorState::Editing(EditingComponent::Tile) => 't',
            EditorState::Editing(EditingComponent::Collider) => 'c',
            _ => {
                '_' //blank here as a fallback to cause a panic
            }
        };

        //update the placeholder object's texture rect to align with the rect given by the event
        for ent in placeholders.iter() {
            commands.entity(ent.0).insert(Sprite {
                image: placeholder.0.clone(),
                rect: Some(Rect {
                    min: Vec2::new(e.rect.min.x, e.rect.min.y),
                    max: Vec2::new(e.rect.max.x, e.rect.max.y),
                }),
                ..default()
            });
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct NormalModeUI;
