use log::error;
use tui::buffer::{Buffer, Cell};
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::StatefulWidget;
use crate::character::Character;
use crate::engine::level::Level;
use crate::map::objects::container::Container;
use crate::map::position::{Area, Position};
use crate::map::tile::TileDetails;
use crate::terminal::colour_mapper;

#[derive(Clone)]
#[derive(Debug)]
pub struct MapWidget {
    level: Level,
    map_display_area : Area // Possibly reduced display area
}

impl MapWidget {
    pub(crate) const fn new(level: Level, map_display_area: Area) -> MapWidget {
        MapWidget { level, map_display_area }
    }

    /*
       Returns true if the given position is inside the range covered by this view
       e.g:
        GIVEN self start_position is x: 5, y: 5 (The map co-ordinate offset)
        AND self view_area is a size of 3 starting from x: 1, y: 1 (this offset is only relevant for display purposed)
        THEN an input Position x 6,7, or 8 would return true (5 + 3 = 8 so range 5-8)
        AND anything above 8 would return false
        AND anything below 5 would return false
    */
    fn is_position_in_map_display_area(&self, position: Position) -> bool {
        self.map_display_area.contains_position(position)
    }

    fn local_to_global(&self, local_area : Area, local_position: Position) -> Option<Position> {

        // Globalise it to a map based co-ord
        let mut globalised_x = local_position.x - local_area.start_position.x;
        let mut globalised_y = local_position.y - local_area.start_position.y;

        // Further globalise it with the map display offset to get the true co-ordinate
        let display_area_start = self.map_display_area.start_position;
        globalised_x = display_area_start.x + globalised_x;
        globalised_y = display_area_start.y + globalised_y;

        let global_position = Position::new(globalised_x, globalised_y);
        return Some(global_position);

        None
    }

    fn global_to_view_local(&self,  local_area : Area, global_position: Position) -> Option<Position> {
            let display_area_start = local_area.start_position;

            // Convert global position to local relative to the display area
            // As display area is also a global position, we can simply get the difference
            // i.e
            // if you're at global position {x: 2, y: 2}
            // and the display area starts at global pos: {x: 1, y: 1}
            // This relative value is {x: 1, y: 1} as you're 1,1 closer to 2,2
            let relative_global_x = global_position.x - display_area_start.x;
            let relative_global_y = global_position.y - display_area_start.y;

            // Apply the view position offsets to this
            // i.e
            // given {x: 1, y: 1}
            // and the view area starts at frame pos {x: 1, y: 1}
            // This gives {x: 2, y: 2}

            let local_x = relative_global_x + local_area.start_position.x;
            let local_y = relative_global_y + local_area.start_position.y;
            let view_position = Position::new(local_x, local_y);
            return Some(view_position);

        None
    }


    fn build_tile_cell(&self, tile_details: &TileDetails) -> Cell {
        let symbol = tile_details.symbol.character.to_string();
        let fg = colour_mapper::map_colour(tile_details.symbol.colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    fn build_character_cell(&self, character: &Character) -> Cell {
        let character_colour = character.get_colour();
        let symbol = character.get_symbol().to_string();
        let fg = colour_mapper::map_colour(character_colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    fn build_container_cell(&self, container: &Container) -> Cell {
        let container_item = container.get_self_item();
        let symbol = container_item.symbol.character.to_string();
        let colour = container_item.symbol.colour;
        let fg = colour_mapper::map_colour(colour);
        let bg = tui::style::Color::Black;
        let modifier = tui::style::Modifier::empty();
        Cell { symbol, fg, bg, modifier }
    }

    fn find_npc(&mut self, global_position: Position) -> Option<Character> {
        let characters = &mut self.level.characters;
        let npcs = characters.get_npcs().clone();
        characters.get_npcs().clone().iter().find(|npc| npc.get_global_position().equals(global_position)).cloned()
    }

    fn find_container(&self, global_position: Position) -> Option<(&Position, &Container)> {
        if let Some(map) = &self.level.map {
            let containers = &map.containers;

            return containers.iter().find(|container_entry| {
                let correct_type = container_entry.1.is_true_container();
                let has_content = container_entry.1.get_total_count() > 0;
                let position_match = container_entry.0.equals(global_position);
                return correct_type && has_content && position_match;
            })
        }
        None
    }
}


impl StatefulWidget for MapWidget {
    type State = MapWidget;

    fn render(mut self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {

        let characters = &mut _state.level.characters;
        let player_mut = characters.get_player_mut().unwrap();
        let map = self.level.map.clone().unwrap();

        // TODO optimise - we should instead go through the items to render
        // and convert global to local positions for better speed
        for x in area.x..area.width {
            for y in area.y..area.height {
                let cell = buf.get_mut(x, y);

                let local_position = Position::new(x,y);
                let global_position = self.local_to_global(Area::from_rect(area), local_position).unwrap();
                if self.is_position_in_map_display_area(global_position) {
                    // Check for the player
                    let player_position = player_mut.get_global_position();
                    if global_position == player_position {
                        let new_cell = self.build_character_cell(player_mut);
                        cell.symbol = new_cell.symbol;
                        cell.fg = new_cell.fg;
                        cell.bg = new_cell.bg;
                    } else if let Some(npc) = self.find_npc(global_position) {
                        let new_cell = self.build_character_cell(&npc);
                        cell.symbol = new_cell.symbol;
                        cell.fg = new_cell.fg;
                        cell.bg = new_cell.bg;
                    } else if let Some(container_entry) = self.find_container(global_position) {
                        let container = container_entry.1;
                        let new_cell = self.build_container_cell(container);
                        cell.symbol = new_cell.symbol;
                        cell.fg = new_cell.fg;
                        cell.bg = new_cell.bg;
                    } else {
                        // Otherwise, just draw the tile
                        let tile_result = map.tiles.get_tile(global_position);
                        if let Some(tile) = tile_result {
                            let new_cell = self.build_tile_cell(&tile);
                            cell.symbol = new_cell.symbol;
                            cell.fg = new_cell.fg;
                            cell.bg = new_cell.bg;
                        }
                    }

                } else {
                    // Draw out of range cell
                    cell.symbol = String::from('?');
                    cell.fg = Color::Red;
                }
            }
        }


    }
}