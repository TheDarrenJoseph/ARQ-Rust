# Known Bugs

Tracking unexpected bugs found during play testing (BUG- prefixed)
Some of these may not be true bugs, but instead needed improvements for playability (PLAY- prefixed)

## [Fixed] BUG-1 - Added Container Items are destroyed upon Container drop into non-room tile

1. GIVEN you've added some items into a container in the player inventory
2. WHEN you drop that container without closing the inventory view first
3. THEN the container is dropped without the newly added items

Fix notes:
We were passing a reference to the same (original) player inventory container to every callback when hooking them up in `inventory_command.open_container(..)`
This meant every time the `CharacterInfoView` fired off a `DropItems` affecting the world state, it would only be using the outdated (original) copy

## [Fixed] BUG-2 Hitting 'c' within a container without having items selected causes a crash

1. GIVEN you're in the container view (of a container on the map, not the player inventory)
2. AND you've not selected anything
3. WHEN you hit 'c' to open the container choice view
4. THEN the game crashes

Fix notes:
Invalid frame size was being built in the calling code (as it's view code it wasn't covered by tests yet).
The building of this also needed to validate it's inputs so we could fail in a sensible way.

Sidenote:
Had to duplicate the usage of try_build_container_choice_frame_handler between character and world container views, 
is there more consolidation we could do for the high-level generic inventory type logic? 

## PLAY-1 - The container choice view is never shown as an option / prompted

It's not clear when you can hit 'c' to use the container choice view

## PLAY-2 Need a way to open parent containers / move up/down the container tree

There's been several adjustments to which container is opened when opening containers in the world..

The current behaviour is that containers in the world are currently opened automatically if:
1. They are the only thing in their parent/floor container
2. They contain items

When dropping non-area containers such as a bag, this behavior prevents you from picking it back up without removing the items inside first, 
or dropping another item in the same space

