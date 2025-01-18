

<H1>Editor Controls</H1>
Editor ![[Pasted image 20250117225456.png]]
**bold means the feature has been implemented already**

Pretty basic so far but this is the idea:
<H3>MODES</H3>
1: **Tilemode**
2:**Collidermode**
3: ...tbd between:  
4: Interactables
5: TriggerBox
6: Flag
7: Wireline (connecting triggers to boxes to flags)
8: Actors
9:
0:...tbd

<H3>Universal Keypresses</H3>
<H4>Tools</H4>
U: Use //uses the current tool basically, this will use the last placed placeholder

P: **(P)lace** *//Note: We should look into making place a repetitive action every few frames if it is still being held down.*
L: **De(L)ete**
M: Recta(m)gle?


WASD: Move Camera


<H3>CTRL Mappings</H3>
CTRL + ?
a:
s: **SaveAsk** (Prompts user if they would like to save the scene, Yenter puts editor into Saving mode which will save the scene, Noscape returns to normal mode without saving)
d:
f:
g:
h:
j:
k:
l: LoadAsk

z:  undo *(probably will not implement, I don't have any good ideas on how to do this)*
x: cut
c: copy *(make the placeholder object the same as whatever is inside the grid space we occupy)*
v: paste *(pretty much just place, I don't think I will implement it any differently than a normal placement of whatever the placeholder object is)*
b: swap between **crosshair**, cursor, and multi modes *(Change the editor to respond to the mouse position or the on-screen crosshair. )*
n: 
m: 

