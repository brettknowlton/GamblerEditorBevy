use bevy::{platform::collections::HashMap, prelude::*};

use super::{anim_def::AnimationDefenition, anim_frame::AnimationFrame, AnimBehavior};

pub struct AnimationMapBuidler {
    map: HashMap<String, AnimationDefenition>,
    spritesheet: Option<Handle<Image>>,
    initial_animation: Option<AnimationDefenition>,
}

impl AnimationMapBuidler {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            spritesheet: None,
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

    /// A helper function to cut a spritesheet into frames based on a standard grid layout,
    /// the frames are cut starting from the top left corner of the spritesheet and going row by row,
    /// the frame size is used to calculate the UV rect for each frame based on the start position and the layout of the frames in the spritesheet
    /// the duration and behavior are the same for all frames in this animation
    pub fn standard_cut(
        mut self,
        name: &str,
        frame_size: Vec2,
        start_frame_index: u32,
        stop_frame_index: u32,
        sheet_layout: Vec2,
        duration: f32,
        behavior: AnimBehavior,
    ) -> Self {
        let mut frames = vec![];

        for y in 0..sheet_layout.y as u32 {
            for x in 0..sheet_layout.x as u32 {
                let new_frame = AnimationFrame::cut_from(
                    sheet_layout,
                    (y * sheet_layout.x as u32 + x) as u64 + start_frame_index as u64,
                    frame_size,
                    duration,
                );
                if (y * sheet_layout.x as u32 + x) as u32 + start_frame_index <= stop_frame_index {
                    frames.push(new_frame);
                } else {
                    continue;
                }
            }
        }
        let actual_frame_count = frames.len() as u32;

        let new_anim = AnimationDefenition {
            frames,
            start_end: Vec2::new(start_frame_index as f32, stop_frame_index as f32),
            frame_count: actual_frame_count,
            current_frame_index: 0,
            frame_timer: 0.0,
            behavior,
            spritesheet: self
                .spritesheet
                .clone()
                .expect("Must set a spritesheet before cutting frames"),
        };

        println!(
            "cut {} frames for animation \"{}\"",
            actual_frame_count, name
        );
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
        self.initial_animation = Some(
            self.map
                .get(name)
                .expect(&format!("Animation map does not contain an animation with the name \"{}\" add it with .add_animation({}, <AnimationDefenition>)", name, name))
                .clone(),
        );
        self
    }

    pub fn build(&self) -> Result<AnimationMap, String> {
        self.check_build_ready();

        if let Some(spritesheet) = &self.spritesheet {
            Ok(AnimationMap {
                current: self.initial_animation.clone(),
                map: self.map.clone(),
                spritesheet: spritesheet.clone(),
            })
        } else {
            Err("Can not create a AnimationMap without a spritesheet. use .with_spritesheet() to set one.".to_string())
        }
    }

    fn check_build_ready(&self) {
        assert!(
            self.spritesheet.is_some(),
            "Must set a spritesheet before building the animation map"
        );
        assert!(
            !self.map.is_empty(),
            "Must add at least one animation before building the animation map"
        );
        assert!(
            self.initial_animation.is_some(),
            "Must set an initial animation before building the animation map"
        );
    }
}

#[derive(Component, Default, Debug, Reflect, Clone)]
pub struct AnimationMap {
    current: Option<AnimationDefenition>,
    map: HashMap<String, AnimationDefenition>,
    pub spritesheet: Handle<Image>,
}

impl AnimationMap {
    pub fn drive(&mut self, dt: Res<Time>) {
        if let Some(current) = &mut self.current {
            current.frame_timer += dt.delta_secs();
            if current.frame_timer >= current.get_current_frame().duration {
                self.next_sprite()
            }
        }
    }

    fn next_sprite(&mut self) {
        if let Some(current) = &mut self.current {
            let frame_count = current.frames.len() as u32;
            if frame_count == 0 {
                return;
            }

            current.frame_timer = 0.0;
            current.current_frame_index += 1;

            if current.current_frame_index >= frame_count {
                match current.behavior {
                    AnimBehavior::Loop => {
                        current.current_frame_index = 0;
                    }
                    AnimBehavior::Once => {
                        current.current_frame_index = frame_count - 1;
                    }
                    AnimBehavior::PingPong => {
                        current.current_frame_index = 0;
                    }
                }
            }
        } else {
            panic!("No current animation set for this AnimationMap");
        }
    }

    pub fn get_current(&self) -> Option<&AnimationDefenition> {
        self.current.as_ref()
    }
}
