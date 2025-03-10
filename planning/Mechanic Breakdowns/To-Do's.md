- [x] implement placeholder as a handle to an image resource, anything that needs to can modify this handle to swap to a different pre-loaded sprite sheet

- [x] every significant comonent needs to:
	- [x] Load a spritesheet into SpriteSheets HashMap. there will be a known limited number of these sprites so maybe a static size HashMap *not needed*
	- [x] there is also now a PlaceholderHandle that keeps the currently used handle. This way we can draw whatever from whatever and it will work out fine.
	- [x] onEnter - make sure visually the sprite changes, we want to use the first TileScaled cut of our spritesheet for the placeholder by default, we can update what cut out with specific systems later.
- [x] Fix Tilemode UI with the spritesheet previews location, anchor Top Left with max width TILE_SCALE * SPRITESHEETWIDTH
- [x] show a grid
- [x] make sure grid works how it should after that implementation and center placements on the crosshair
- [ ] make snap to grid toggleable  (maybe ctrl+g?)

- [ ] more bottom bar UI
	- [ ] think about showing controls?

- [ ] look into mappable controls.
	- [ ] hashmap of Action String -> fnmut?? that would be cool but a lot of work, FnMut() is a bitch tho with | | syntax and shit

- [ ] Next we can implement rectangle tool for SelectionRect, SelectionRect will be our test case for this kind of functionality. - actual selection functionality we will not worry about yet
- [ ] 
- [ ] make sure selection rect works with keybinds
- [ ] work on making a "using_tool" idea where the last used tool maps to "U" and we can re-use that at any point- this will lead into mouse funcitonality
- [ ] implement colliders:, sprites for colliders, and verify colliders work
- [ ] implement rectangle tool for colliders
- [ ] rework spawn_sprites to handle all cases of spritesheets not just
- [ ] spend some time making spritesheets feel actually nice
- [ ] tilemodeUI, I want the selectionRect in the UI to grow and move dynamically in the spritesheet, wrapping around if neccisarry. 
	- [ ] We can create 2- rects if needed with functions to create a secondary rect if needs be to cover the "wrapped" area on the other side of the spritesheet, until the current rect's min is wrapped around as well.