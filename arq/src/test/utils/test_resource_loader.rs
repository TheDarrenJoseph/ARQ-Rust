use rand_seeder::Seeder;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use tui::buffer::Buffer;
use uuid::Uuid;
use crate::character::builder::character_builder::{CharacterBuilder, CharacterPattern};
use crate::character::characters::Characters;
use crate::character::Character;
use crate::engine::level::{init_level_manager, Level, Levels};
use crate::map::objects::container::{Container, ContainerType};
use crate::map::position::{build_square_area, Area, Position};
use crate::map::tile::{Colour, TileType};
use crate::map::{Map, Tiles};
use crate::map::objects::items::Item;

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