ASCII Roguelike Quester (Rust Port) [Rust]
=============================================================

A port of https://github.com/TheDarrenJoseph/ARQ to Rust


![Build Status](https://github.com/TheDarrenJoseph/ARQ-Rust/actions/workflows/main-build.yml/badge.svg)

INSPIRATION
=======
- Nethack
- Dwarf Fortress

MADE USING
=======
- [tui](https://github.com/fdehau/tui-rs)

TODO (Not in any specific order)
----
- [ ] Procedural levels
    - [X] Room generation
    - [X] Pathfinding
    - [ ] Level traversal (Exit/Entry, next level/previous level)
- [ ] Overworld
- [ ] Items (valuables, potions, scrolls, etc)
    - [X] Basic valuables
    - [ ] Usable items
    - [ ] Equippable / Equipment (Armour, Weapons, etc)
    - [X] Containers / Container spawning
    - [X] World container view / handling (i.e Floor, Chests)
        - [X] Dropping items / multiple items in one spot
        - [X] Taking items
        - [X] Moving items inplace
        - [X] Moving between containers
    - [ ] Inventory Tab (Character Info View)
        - [X] Dropping items
        - [X] Moving items in-place
        - [X] Moving between containers
        - [ ]  'Use' for items (Nothing usable yet)
        - [ ]  'Equip' for items (Nothing equippable yet)
- [ ] Character Stats (For player and NPCs)
    - [X] Character creation
    - [ ] NPC Character creation
- [ ] Character Tab (Character Info View)
    - [ ] Stats view
    - [ ] Leveling / Stat edit
- [ ] Health System
- [ ] Combat system
- [ ] NPCs
    - [ ] Spawning
    - [ ] Basic pathfinding / player seeking
    - [ ] Combat turns
    - [ ] Level bosses
- [X] Settings Menu
- [ ] Ending Screen (Game Over / Dungeon escape)
- [ ] Leaderboard / Graveyard
- [ ] (Optional) Export / Load game/item maps from files
