use bevy::{platform::collections::HashMap, prelude::*};

use super::{anim_def::AnimationDefenition, AnimBehavior};

pub struct AnimationMapBuidler {
    map: HashMap<String, AnimationDefenition>,
    spritesheet: Option<Handle<Image>>,
    atlas_layout: Option<Handle<TextureAtlasLayout>>,
    initial_animation: Option<String>,
}

impl AnimationMapBuidler {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            spritesheet: None,
            atlas_layout: None,
            initial_animation: None,
        }
    }

    pub fn with_spritesheet_path(mut self, asset_server: &AssetServer, tex_path: String) -> Self {
        self.spritesheet = Some(asset_server.load(tex_path.clone()));
        self
    }
    pub fn with_spritesheet(mut self, spritesheet: Handle<Image>) -> Self {
        self.spritesheet = Some(spritesheet);
        self
    }

    pub fn with_atlas_layout(mut self, atlas_layout: Handle<TextureAtlasLayout>) -> Self {
        self.atlas_layout = Some(atlas_layout);
        self
    }

    /// A helper function to cut a spritesheet into frames based on a standard grid layout,
    /// the frames are cut starting from the top left corner of the spritesheet and going row by row,
    /// the frame size is used to calculate the UV rect for each frame based on the start position and the layout of the frames in the spritesheet
    /// the duration and behavior are the same for all frames in this animation
    pub fn standard_cut(
        mut self,
        name: &str,
        _frame_size: Vec2,
        start_frame_index: u32,
        stop_frame_index: u32,
        _sheet_layout: Vec2,
        duration: f32,
        behavior: AnimBehavior,
    ) -> Self {
        if self.atlas_layout.is_none() {
            warn!(
                "No atlas layout set before standard_cut for animation '{}' - call with_atlas_layout first",
                name
            );
            return self;
        }

        if stop_frame_index < start_frame_index {
            warn!(
                "Ignoring animation '{}' because stop_frame_index ({}) is less than start_frame_index ({})",
                name, stop_frame_index, start_frame_index
            );
            return self;
        }

        let actual_frame_count = stop_frame_index - start_frame_index + 1;

        let new_anim = AnimationDefenition {
            start_index: start_frame_index,
            stop_index: stop_frame_index,
            current_frame_index: 0,
            frame_timer: 0.0,
            frame_duration: duration.max(0.001),
            behavior,
            pingpong_forward: true,
        };

        debug!("Registered animation '{}' with {} frames", name, actual_frame_count);
        self.map.insert(name.to_string(), new_anim);
        self
    }

    pub fn add_animation(mut self, name: &str, defenition: AnimationDefenition) -> Self {
        self.map.insert(name.to_string(), defenition);
        self
    }

    pub fn set_initial_animation(mut self, name: &str) -> Self {
        if self.initial_animation.is_some() {
            warn!("Initial animation is already set");
        }
        if !self.map.contains_key(name) {
            warn!(
                "Animation map does not contain animation '{}' (add it before set_initial_animation)",
                name
            );
            return self;
        }
        self.initial_animation = Some(name.to_string());
        self
    }

    pub fn build(&self) -> Result<AnimationMap, String> {
        self.check_build_ready()?;

        let spritesheet = self
            .spritesheet
            .clone()
            .ok_or_else(|| "Can not create AnimationMap without spritesheet".to_string())?;
        let atlas_layout = self
            .atlas_layout
            .clone()
            .ok_or_else(|| "Can not create AnimationMap without atlas layout".to_string())?;
        let current = self.initial_animation.clone();

        Ok(AnimationMap {
            current,
            map: self.map.clone(),
            spritesheet,
            atlas_layout,
        })
    }

    fn check_build_ready(&self) -> Result<(), String> {
        if self.spritesheet.is_none() {
            return Err(
                "Must set a spritesheet before building the animation map (use .with_spritesheet())"
                    .to_string(),
            );
        }
        if self.map.is_empty() {
            return Err("Must add at least one animation before building the animation map".to_string());
        }
        if self.initial_animation.is_none() {
            return Err("Must set an initial animation before building the animation map".to_string());
        }
        if self.atlas_layout.is_none() {
            return Err(
                "Must set an atlas layout before building (use .with_atlas_layout() or .standard_cut())"
                    .to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Component, Default, Debug, Reflect, Clone)]
pub struct AnimationMap {
    current: Option<String>,
    map: HashMap<String, AnimationDefenition>,
    pub spritesheet: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

impl AnimationMap {
    pub fn drive(&mut self, dt_secs: f32) {
        let Some(current_name) = self.current.clone() else {
            return;
        };

        let mut should_advance = false;
        if let Some(current) = self.map.get_mut(&current_name) {
            current.frame_timer += dt_secs;
            if current.frame_timer >= current.frame_duration {
                current.frame_timer = 0.0;
                should_advance = true;
            }
        }

        if should_advance {
            self.next_sprite();
        }
    }

    fn next_sprite(&mut self) {
        let Some(current_name) = self.current.clone() else {
            return;
        };

        let Some(current) = self.map.get_mut(&current_name) else {
            return;
        };

        let frame_count = current.frame_count();
        if frame_count <= 1 {
            current.current_frame_index = 0;
            return;
        }

        match current.behavior {
            AnimBehavior::Loop => {
                current.current_frame_index = (current.current_frame_index + 1) % frame_count;
            }
            AnimBehavior::Once => {
                if current.current_frame_index + 1 < frame_count {
                    current.current_frame_index += 1;
                }
            }
            AnimBehavior::PingPong => {
                if current.pingpong_forward {
                    if current.current_frame_index + 1 >= frame_count {
                        current.pingpong_forward = false;
                        current.current_frame_index = current.current_frame_index.saturating_sub(1);
                    } else {
                        current.current_frame_index += 1;
                    }
                } else if current.current_frame_index == 0 {
                    current.pingpong_forward = true;
                    current.current_frame_index = 1;
                } else {
                    current.current_frame_index -= 1;
                }
            }
        }
    }

    pub fn get_current(&self) -> Option<&AnimationDefenition> {
        let current_name = self.current.as_ref()?;
        self.map.get(current_name)
    }

    pub fn get_current_name(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn get_current_atlas_index(&self) -> Option<usize> {
        self.get_current().map(AnimationDefenition::current_atlas_index)
    }

    pub fn set_current_animation(&mut self, name: &str, restart: bool) -> Result<bool, String> {
        if !self.map.contains_key(name) {
            return Err(format!("Animation '{}' does not exist in map", name));
        }

        if self.current.as_deref() == Some(name) {
            if restart {
                if let Some(current) = self.map.get_mut(name) {
                    current.reset();
                }
            }
            return Ok(false);
        }

        self.current = Some(name.to_string());
        if let Some(current) = self.map.get_mut(name) {
            current.reset();
        }

        Ok(true)
    }
}
