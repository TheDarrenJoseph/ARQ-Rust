use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use rand_seeder::Seeder;

use tui::buffer::Buffer;

use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern};
use crate::character::Character;
use crate::character::characters::Characters;
use crate::engine::level::{init_level_manager, Level, Levels};
use crate::map::objects::container::Container;
use crate::map::position::{Area, build_square_area, Position};
use crate::map::tile::TileType;
use crate::map::{Map, Tiles};

mod text_widget_tests;
mod dropdown_widget_tests;
mod number_widget_tests;

pub fn read_expected_buffer_file(path: String, buffer_area: Area) -> Buffer {
    let mut input_string = String::new();
    File::open(path.clone()).unwrap().read_to_string(&mut input_string).expect(format!("The file '{}' should have been read to String", path).as_str());

    let mut lines = Vec::new();
    input_string.lines().for_each(|l| lines.push(l));
    let mut buffer_lines: Vec<String> = Vec::new();
    for y in 0..buffer_area.height as usize {
        let line = lines.get(y).expect(format!("File lines should contain an index of: {}", y).as_str());
        let line_string = String::from(*line);
        buffer_lines.push(String::from(line_string))
    }

    Buffer::with_lines(buffer_lines)
}

pub fn build_test_level(area_container: Option<(Position, Container)>, player: Option<Character>) -> Level {
    let tile_library = crate::map::tile::build_library();
    let rom = tile_library[&TileType::Room].clone();
    let wall = tile_library[&TileType::Wall].clone();
    let map_pos = Position { x: 0, y: 0 };
    let map_area = build_square_area(map_pos, 3);
    
    // Add the custom area container at pos 0,0 if provided
    let mut area_containers = HashMap::new();
    if let Some(c) = area_container {
        area_containers.insert(c.0, c.1);
    }

    let map = crate::map::Map {
        area: map_area,
        tiles : Tiles { tiles : vec![
            vec![ wall.clone(), wall.clone(), wall.clone() ],
            vec![ wall.clone(), rom.clone(), wall.clone() ],
            vec![ wall.clone(), wall.clone(), wall.clone() ],
        ]},
        rooms: Vec::new(),
        containers: area_containers
    };


    let player_choice = if let Some(p) = player {
        p
    } else {
        let player_pattern_result = CharacterPattern::new_player();
        assert!(player_pattern_result.is_ok(), "Failed to build player CharacterPattern!");
        CharacterBuilder::new(player_pattern_result.unwrap())
            .build(String::from("Test Player"))
    };
    
    return  Level { map: Some(map) , characters: Characters::new( Some(player_choice), Vec::new())  };
}

pub fn build_test_levels(map: Map, player: Character) -> Levels {
    build_test_levels_for_level(Level {
        map: Some(map.clone()),
        characters: Characters::new(Some(player), Vec::new())
    })
}

pub fn build_test_levels_for_level(level: Level) -> Levels {
    let seed = "test".to_string();
    let seed_copy = seed.clone();
    let rng = Seeder::from(seed).make_rng();
    let mut levels = init_level_manager(seed_copy, rng);
    levels.add_level_directly(level);
    levels
}
