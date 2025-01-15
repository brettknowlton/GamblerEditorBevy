walkthrough logic:
press a key "r", on the keypress it will spawn a rect with one corner on the location pressed and another corner always moving to the "selected_point_position" which may need to be a resource? you cant have more than one selected point so that would probably work

next the idea would be on the release of that key: listen for the key to be released and then spawn a "selectedArea" entity that uses the keypressed point and the keyreleassed point as it's bounding corners.

"mode" enum should maybe be tied into types generically if I can make it work- I would like this rectangle to be implemented for EACH EditorObject type, maybe it would be worth while to implemet a GameObject Trait to accomplish this.

	fn keybinds(...){
		...
		if 'r' on_press{
			commands.spawn(SelectedArea)With Tracking:True or something
			basically this is the same entity as it will be for eternity, the keyrelease just makes it so we don't keep following it
		}
	
		if 'r' on release
			commands.entity(e).whatever the fuck i dont remember- basically we are going to overwrite the SelectedArea to have tracking:false. Once tracking is set false on a rectangle it will be implemented so you can not change it again (easily, maybe detailed editing would be good idk)
		...
	}


	fn fill_selections(...) {
		any time we have a fillable selection, fill it with the identifying type of whatever our selected placement is going to be.
		match get_state(get_state(EditorMode)){
			Editing{EditingMode::Tile} {
				fill_selection::<Tile>()
			},
			...etc(),
			All modes that are not related to Editing we will not kill the tile like usual but flip a different flag of "fillable" to false. This will leave the selection alive at the end of this function and allow us to get a [[#Context Menu]] on it later if we want to "ask the user" how the context menu will be used
		}
		
		fill_selection::<>() will be Editing(EditingMode::Something) and change to something.get_identifying_type
	idea being that fill_selection will be implemented for any indentifying type in such a way that the object could be filled into that selection (or print appropriate errors)
	}