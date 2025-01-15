
Game objects "right click" options. Each can implement their own with a trait if I do this right

trait Contextable{
	//Something like an event here maybe? Events that this object *COULD* call?
	get_options() -> Vec<>;
	
	
	display_options();
}