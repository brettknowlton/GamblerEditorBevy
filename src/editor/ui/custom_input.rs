use bevy::input::keyboard::KeyCode;
use bevy_egui::egui::{self, TextureId};
use bevy::prelude::*;

#[derive(Default, Clone, Debug)]
pub enum CustomInput {
    #[default]
    Empty,
    Single(KeyCode),
    Combo(Vec<KeyCode>),
    Multi(Vec<KeyCode>),
}

impl CustomInput {
    fn simplify_key(kc: &KeyCode) -> String {
        let key_str = format!("{:?}", kc);

        let mut raw_key: String = "".into();
        if key_str.contains("Key") {
            raw_key = key_str.replace("Key", "");
        }

        format!("{}", raw_key)
    }

    fn keybind_hint_text(input_type: &CustomInput) -> String {
        match input_type {
            CustomInput::Single(key_code) => CustomInput::simplify_key(key_code),
            CustomInput::Multi(key_codes) => CustomInput::simplify_key(key_codes.first().unwrap()),
            CustomInput::Combo(key_codes) => {
                let letters: Vec<&KeyCode> = key_codes
                    .iter()
                    .filter(|value| **value != KeyCode::ControlLeft)
                    .collect();

                return format!("^{}", CustomInput::simplify_key(letters.first().unwrap()));
            }
            CustomInput::Empty => String::new(),
        }
    }
}

#[derive(Default, Clone)]
pub struct CustomInputAction {
    input_type: CustomInput,
    action: String,
}

impl CustomInputAction {
    pub fn show(&self, ui: &mut egui::Ui, texture_id: TextureId) {
        let icon_size = 20.0;

        ui.add(egui::Label::new(
            egui::RichText::new(self.action.as_str())
                .size(16.0)
                .color(egui::Color32::from_rgba_unmultiplied(220, 230, 245, 255)),
        ));

        let image = egui::widgets::Image::new(egui::load::SizedTexture::new(
            texture_id,
            [icon_size, icon_size],
        ));

        let rect = ui.add(image).rect;
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            CustomInput::keybind_hint_text(&self.input_type),
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(18, 22, 30),
        );
    }
}

#[derive(Resource, Default)]
pub struct AvailableKeybinds {
    pub keybinds: Vec<CustomInputAction>,
}

impl AvailableKeybinds {
    fn add(&mut self, input_action: CustomInputAction) {
        self.keybinds.append(&mut vec![input_action]);
    }

    pub fn add_keycode(&mut self, input: CustomInput, action: String) {
        let input = CustomInputAction {
            input_type: input,
            action: action,
        };

        self.add(input);
    }

    pub fn remove_kb(&mut self, kb_id: usize) {
        self.keybinds.remove(kb_id);
    }

    pub fn clear(&mut self) {
        self.keybinds = vec![];
    }
}
