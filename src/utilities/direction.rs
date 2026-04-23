use super::*;

#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
