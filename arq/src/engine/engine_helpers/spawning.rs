/*
 * Finds the first entry or exit containing room depending on the direction
 * Sets the player position to match that
 * Returns the room the player has been moved to (for further spawning decisions)
 */
use rand::{thread_rng, Rng};

use crate::engine::game_engine::GameEngine;
use crate::engine::level::LevelChange;
use crate::map::room::Room;
use crate::util::utils::UuidEquals;

pub fn respawn_player<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>, change: LevelChange) -> Option<Room> {
    let level = engine.levels.get_level_mut();
    let player = level.characters.get_player_mut().unwrap();

    // Grab the first room and set the player's position there
    if let Some(map) = &level.map {
        match change {
            LevelChange::UP => {
                let exit_room = map.rooms.iter().find(|room| room.get_exit().is_some()).unwrap();
                player.set_position(exit_room.get_exit().unwrap());
                return Some(exit_room.clone());
            },
            LevelChange::DOWN => {
                let entry_room = map.rooms.iter().find(|room| room.get_entry().is_some());
                if let Some(er) = entry_room {
                    player.set_position(er.get_entry().unwrap());
                    return Some(er.clone());
                }
            },
            _ => { }
        }
    } else {
        log::error!("Cannot respawn player, Map was None!");
    }
    return None;
}

pub fn respawn_npcs<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>, player_room: Room) {
    let level = engine.levels.get_level_mut();
    let npcs = level.characters.get_npcs_mut();
    if let Some(map) = &level.map {
        let mut non_player_rooms : Vec<Room> = map.rooms.clone();
        non_player_rooms.retain(|r| !r.uuid_equals(player_room.clone()));

        if !non_player_rooms.is_empty() {
            let _moved = 0;
            for npc in npcs {
                // Normal thread RNG / non-reproducible!!
                let mut rng = thread_rng();
                let random_room_idx = rng.gen_range(0..non_player_rooms.len() - 1);
                let chosen_room = non_player_rooms.get(random_room_idx).unwrap();
                npc.set_position(chosen_room.random_inside_pos(&mut rng));
            }
        } else {
            log::error!("Cannot respawn NPCs, Cannot find any non player containing rooms.");
        }
    } else {
        log::error!("Cannot respawn NPCs, Map was None!");
    }
}
