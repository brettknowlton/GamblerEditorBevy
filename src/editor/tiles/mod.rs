use crate::utilities::Coordinate;


pub struct Tile {
    pub tile_type: u64,
    pub coordinate: Coordinate,
    pub sprite: Sprite,
}

/// R , G, B
pub struct Sprite(u64, u64, u64);


impl Tile {
    pub fn new() -> Self {
        Self {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
            sprite: Sprite(255, 255, 255),
        }
    }

    /** 
     * Takes a coordinate of the currently 
     * selected tile via the hovered crosshair 
     * and adds it to the scene.
     */
    fn place(&self, coordinate: Coordinate, id: u64) {
        todo!()
    }

    fn get_coordinate(&self) -> Coordinate {
        todo!()
    }

    /**
     * This will be a file that we will parse through 
     * the file at a given tile size some sprites,
     * we will ID them based on order received, which will 
     * not ever change throughout the game unless we want to 
     * change what the tile looks like
     */
    fn import_sprite(&mut self) -> anyhow::Result<Sprite, anyhow::Error> {
        todo!()
    }

}