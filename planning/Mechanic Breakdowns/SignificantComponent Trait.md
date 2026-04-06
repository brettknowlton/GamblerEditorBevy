Significant components are the "Types of things we are actually placing with the editor"
The unique component that each plugin is built around should impl the SignificantComponent Trait so we can do things like place, remove, use the rectangle tool, or any other kind of use case we want to add down the road.

This will be the start of generalizing our editor so building from here we should see more types of things popping up


functions of the SignificantComponent trait would include:
place()//places this component in the world aligned to the grid
remove()//removes the component occupying this coordinate space

place_rectangle(rect:: Rect)//place this significant component using the provided rect. see [[Rectangle Tool]]
open_context()//open the context menu on this significant component. see[[Context Menu]]