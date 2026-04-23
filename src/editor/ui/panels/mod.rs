use bevy_egui::egui;

pub mod bottom_panel;
pub mod left_panel;
pub mod mode_tabs_panel;
pub mod right_tools_panel;

pub use bottom_panel::BottomPanel;
pub use left_panel::LeftPanel;
pub use mode_tabs_panel::ModeTabsPanel;
pub use right_tools_panel::RightToolsPanel;

pub fn editor_panel_frame() -> egui::Frame {
    egui::Frame {
        fill: egui::Color32::from_rgba_unmultiplied(18, 22, 30, 165),
        stroke: egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(220, 230, 245, 120),
        ),
        inner_margin: egui::Margin::same(8),
        ..Default::default()
    }
}
