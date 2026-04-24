use bevy_egui::EguiContexts;

use crate::{
    editor_modes::EditorObjectKind, TextureHandles, SCALED_TILE_HEIGHT, SCALED_TILE_WIDTH,
};

use super::*;

#[derive(Component)]
pub struct DisplayMessage;

pub fn configure_tooling_menu(
    tooling_menu: &mut ToolingMenuState,
    title: &str,
    selected_item_id: Option<u64>,
    items: Vec<ToolingMenuItem>,
) {
    tooling_menu.title = title.to_string();
    tooling_menu.visible = true;
    tooling_menu.selected_item_id = selected_item_id;
    tooling_menu.items = items;
}

#[derive(Clone, Debug, Default)]
pub struct ToolingMenuItem {
    pub id: u64,
    pub label: String,
    pub texture_key: Option<EditorObjectKind>,
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

#[derive(Resource, Default)]
pub struct LeftPanelEdge(pub f32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum MouseToolKind {
    #[default]
    Pointer,
    Eyedropper,
}

#[derive(Resource, Debug, Default)]
pub struct MouseToolState {
    pub current: MouseToolKind,
}

pub fn render_egui_panels(
    mut contexts: EguiContexts,

    bottom_bar: Res<message_display::MessageDisplay>,
    available_keybinds: Res<AvailableKeybinds>,
    asset_server: Res<AssetServer>,

    editor_state: Res<State<EditorState>>,
    input: Res<ButtonInput<KeyCode>>,
    tooling_menu_state: ResMut<ToolingMenuState>,
    textures: Res<TextureHandles>,
    mut next_editor_state: ResMut<NextState<EditorState>>,
    mut left_panel_edge: ResMut<LeftPanelEdge>,
    mut mouse_tool_state: ResMut<MouseToolState>,

    mut bottom_panel: ResMut<BottomPanel>,
    mut left_panel: ResMut<LeftPanel>,
    mut mode_tabs_panel: ResMut<ModeTabsPanel>,
    mut right_tools_panel: ResMut<RightToolsPanel>,
) -> Result {
    bottom_panel.show(
        &mut contexts,
        bottom_bar.as_ref(),
        available_keybinds.as_ref(),
        asset_server.as_ref(),
    )?;

    let ctx = contexts.ctx_mut()?;
    mode_tabs_panel.show(
        ctx,
        editor_state.get(),
        &mut next_editor_state,
        left_panel_edge.0,
    );

    let panel_right_x = left_panel.show(
        &mut contexts,
        &editor_state,
        tooling_menu_state,
        &input,
        &textures,
    )?;

    left_panel_edge.0 = panel_right_x;

    right_tools_panel.show(
        &mut contexts,
        asset_server.as_ref(),
        editor_state.get(),
        &mut mouse_tool_state,
    )?;

    Ok(())
}

///Systems have been added for this component to keep all UI items moving at the same speed, and therefore always relatively positioned to eachother.
/// Useful for menus, or any thing that you want to keep moving based on the camera's location. This does not prevent movement of the object by other systems,
/// we are just also using this to TAG all UI items so we can easily find them in queries (typically for movement so far)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform)]
pub struct CameraLockedUI {
    pub vel_x: f32,
    pub vel_y: f32,
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
