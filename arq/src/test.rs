use std::fmt::format;
use std::fs::File;
use std::io;
use std::io::Read;
use tui::buffer::Buffer;
use tui::style::{Modifier, Style};
use crate::map::position::Area;

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

    let mut expected = Buffer::with_lines(buffer_lines);
    expected
}