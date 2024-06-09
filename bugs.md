# Known Bugs

Tracking unexpected bugs found during play testing (BUG- prefixed)
Some of these may not be true bugs, but instead needed improvements for playability (PLAY- prefixed)

## BUG-1 - Added Container Items are destroyed upon Container drop into non-room tile

GIVEN you've added some items into a container in the player inventory
WHEN you drop that container onto a non-room tile (i.e the corridor tile)
THEN the container is dropped without the newly added items

## BUG-2 Hitting 'c' within a container without having items selected causes a crash

GIVEN you're in the container view (of a container on the map, not the player inventory)
AND you've not selected anything
WHEN you hit 'c' to open the container choice view
THEN the game crashes

## PLAY-1 - The container choice view is never shown as an option / prompted

It's not clear when you can hit 'c' to use the container choice view

## PLAY-1 Need a way to open parent containers / move up/down the container tree

There's been several adjustments to which container is opened when opening containers in the world..

The current behaviour is that containers in the world are currently opened automatically if:
1. They are the only thing in their parent/floor container
2. They contain items

When dropping non-area containers such as a bag, this behavior prevents you from picking it back up without removing the items inside first, 
or dropping another item in the same space

