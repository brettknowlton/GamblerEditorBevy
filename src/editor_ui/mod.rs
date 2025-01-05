use crate::{actors::Actor, interactables::Interactable, tiles::Tile, utilities::Coordinate};
use std::collections::HashMap;

/// Game objects / location when loaded into the game
pub struct Scene {
    tiles: HashMap<Coordinate, Tile>,
    interactables: HashMap<Coordinate, Interactable>,
    actors: HashMap<Coordinate, Actor>
}
