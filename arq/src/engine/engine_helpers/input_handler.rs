use std::io::Error;
use termion::event::Key;
use crate::engine::command::command::Command;
use crate::engine::command::input_mapping;
use crate::engine::command::inventory_command::InventoryCommand;
use crate::engine::command::look_command::LookCommand;
use crate::engine::command::open_command::OpenCommand;
use crate::engine::engine_helpers::menu::menu_command;
use crate::engine::game_engine::GameEngine;
use crate::view::game_over_view::GameOverChoice;

pub async fn handle_input<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>, key : Key) -> Result<Option<GameOverChoice>, Error>  {

    let level = engine.levels.get_level_mut();
    let ui_wrapper = &mut engine.ui_wrapper;
    match key {
        Key::Esc => {
            if let Some(goc) = menu_command(engine).await? {
                return Ok(Some(goc));
            }
        },
        Key::Char('c') => {
            engine.begin_combat()?;
        },
        Key::Char('i') => {
            let mut command = InventoryCommand {
                level,
                ui: &mut engine.ui_wrapper.ui,
                terminal_manager: &mut engine.ui_wrapper.terminal_manager
            };
            command.handle(key)?;
        },
        Key::Char('k') => {
            let mut command = LookCommand {
                level,
                ui: &mut engine.ui_wrapper.ui,
                terminal_manager: &mut engine.ui_wrapper.terminal_manager
            };
            command.handle(key)?;
        },
        Key::Char('o') => {
            let key = ui_wrapper.get_prompted_input(String::from("What do you want to open?. Arrow keys to choose. Repeat usage to choose current location."))?;
            let mut command = OpenCommand {
                level,
                ui: &mut engine.ui_wrapper.ui,
                terminal_manager: &mut engine.ui_wrapper.terminal_manager
            };
            command.handle(key)?;
        },
        Key::Down | Key::Up | Key::Left | Key::Right | Key::Char('w') | Key::Char('a') | Key::Char('s') | Key::Char('d') => {
            if let Some(side) = input_mapping::key_to_side(key) {
                if let Some(game_over_choice) = engine.handle_player_movement(side).await? {
                    return Ok(Some(game_over_choice));
                }
            }
        },
        _ => {}
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rand_seeder::Seeder;
    use termion::event::Key;
    use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern};

    use crate::character::Character;
    use crate::character::characters::Characters;
    use crate::engine::engine_helpers::input_handler::handle_input;

    use crate::engine::game_engine::*;
    use crate::engine::level::{init_level_manager, Level, Levels};
    use crate::map::{Map, Tiles};
    use crate::map::position::{Position};
    use crate::map::position::{Area, build_square_area};
    use crate::map::tile::{build_library, TileType, TileDetails};
    use crate::terminal::terminal_manager;

    fn build_tiles(map_area: Area, tile : TileType) -> Vec<Vec<TileDetails>> {
        let tile_library = build_library();
        let mut map_tiles = Vec::new();
        let mut row;
        for _y in map_area.start_position.y..=map_area.end_position.y {
            row = Vec::new();
            for _x in map_area.start_position.x..=map_area.end_position.x {
                row.push( tile_library[&tile].clone());
            }
            map_tiles.push(row);
        }
        map_tiles
    }

    async fn test_movement_input(levels: Levels, start_position: Position, input: Vec<Key>, expected_end_position : Position) {
        // GIVEN a game engine with a 3x3 grid of tiles
        let _tile_library = build_library();
        let terminal_manager = terminal_manager::init_test().unwrap();
        let game_engine = build_test_game_engine(levels, terminal_manager);

        match game_engine {
            Ok(mut engine) => {
                let levels = engine.levels.get_level_mut();
                let player = levels.characters.get_player_mut();
                assert_eq!(start_position, player.unwrap().get_global_position());

                for key in input {
                    handle_input(&mut engine, key).await.unwrap();
                }
                assert_eq!(expected_end_position, engine.levels.get_level_mut().characters.get_player().unwrap().get_global_position());
            },
            _ => {
                panic!("Expected a valid Game Engine instance!")
            }
        }
    }

    fn build_test_levels(map: Map, player: Character) -> Levels {
        let level = Level {
            map: Some(map.clone()),
            characters: Characters::new(Some(player), Vec::new())
        };

        let seed = "test".to_string();
        let seed_copy = seed.clone();
        let rng = Seeder::from(seed).make_rng();
        let mut levels = init_level_manager(seed_copy, rng);
        levels.add_level_directly(level);
        levels
    }

    #[test]
    fn test_build_game_engine() {
        let terminal_manager = terminal_manager::init_test().unwrap();
        let _game_engine = build_game_engine(terminal_manager);
    }

    #[tokio::test]
    async fn test_move_down_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND the middle / bottom middle tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        map_tiles[2] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};

        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");

        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

    #[tokio::test]
    async fn test_move_down_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND only the middle tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");

        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);
        // WHEN we attempt to move down
        let input = vec![Key::Down];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

    #[tokio::test]
    async fn test_move_up_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND the middle / bottom middle tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        map_tiles[2] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles },
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the bottom middle of the map
        let start_position = Position{x:1, y:2};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

    #[tokio::test]
    async fn test_move_up_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[2] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle end of the map
        let start_position = Position{x:1, y:2};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move up
        let input = vec![Key::Up];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 2};
        test_movement_input(levels, start_position, input, expected_end_position);
    }

    #[tokio::test]
    async fn test_move_left_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND the middle / middle left tile is a corridor
        map_tiles[1] [0] = tile_library[&TileType::Corridor].clone();
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:0, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position);
    }

    #[tokio::test]
    async fn test_move_left_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move left
        let input = vec![Key::Left];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

    #[tokio::test]
    async fn test_move_right_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND the middle / middle right tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        map_tiles[1] [2] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to be updated to the other corridor tile
        let expected_end_position = Position{x:2, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

    #[tokio::test]
    async fn test_move_right_non_traversable() {
        // GIVEN a game engine with a 3x3 grid of tiles
        let tile_library = build_library();
        let map_area = build_square_area(Position { x: 0, y: 0}, 3);
        let mut map_tiles = build_tiles(map_area.clone(), TileType::NoTile);

        // AND only the middle end tile is a corridor
        map_tiles[1] [1] = tile_library[&TileType::Corridor].clone();
        let map = Map {
            area: map_area,
            tiles : Tiles { tiles: map_tiles},
            rooms : Vec::new(),
            containers: HashMap::new()
        };

        // AND the player start position is the middle of the map
        let start_position = Position{x:1, y:1};
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        let player =  CharacterBuilder::new(player_pattern_result.unwrap())
            .position(start_position)
            .build(String::from("Test Player"));
        let levels = build_test_levels(map, player);

        // WHEN we attempt to move right
        let input = vec![Key::Right];
        // THEN we expect the player's position to remain unchanged
        let expected_end_position = Position{x:1, y: 1};
        test_movement_input(levels, start_position, input, expected_end_position).await;
    }

}