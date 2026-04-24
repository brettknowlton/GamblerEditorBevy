use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Reflect, Debug, Clone, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum TileID {
    #[default]
    Any,
    None,
    Some(u64),
}

impl PartialEq for TileID {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Rule 1: All Some are if their inner values are equal
            (Self::Some(_), Self::Some(_)) => true,

            // Rule 2: Any are always equal
            (Self::Any, Self::Any) => true,

            // Rule 3: None are always equal
            (Self::None, Self::None) => true,

            // All other combinations are false
            _ => false,
        }
    }
}
