use log::{debug, error, info};
use tui::buffer::{Buffer, Cell};
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::StatefulWidget;
use crate::character::Character;
use crate::engine::level::Level;
use crate::map::Map;
use crate::map::map_view_areas::MapViewAreas;
use crate::map::objects::container::Container;
use crate::map::position::{Area, Position};
use crate::map::tile::TileDetails;
use crate::terminal::colour_mapper;
use crate::view::map_view::{MapView};
use crate::view::util::cell_builder::CellBuilder;

#[derive(Clone)]
#[derive(Debug)]
pub struct MapWidget {
    map_view_areas : MapViewAreas // Possibly reduced display area
}

impl MapWidget {
    pub(crate) const fn new(map_view_areas: MapViewAreas) -> MapWidget {
        MapWidget { map_view_areas }
    }


    fn find_container<'a>(&'a self, map: &'a Map, global_position: Position) -> Option<(&Position, &Container)> {
        let containers = &map.containers;

        return containers.iter().find(|container_entry| {
            let correct_type = container_entry.1.is_true_container();
            let has_content = container_entry.1.get_total_count() > 0;
            let position_match = container_entry.0.equals(global_position);
            return correct_type && has_content && position_match;
        });
        None
    }

    fn build_cell_for_position(&mut self, level: &mut Level, global_position: Position, mut cell_target: &mut Cell) -> Cell {
        let characters = &mut level.characters;
        let player_mut = characters.get_player_mut().unwrap();

        if let Some(map) = &level.map {
            let tiles = &map.tiles;
            let tile_result = tiles.get_tile(global_position);

            // Check for the player
            if global_position == player_mut.get_global_position() {
                return CellBuilder::from_character(player_mut);
            }

            let characters = &mut level.characters;
            if let Some(npc) = characters.get_npcs().iter().find(|npc| npc.get_global_position().equals(global_position)).cloned() {
                return CellBuilder::from_character(&npc);
            }

            if let Some(container_entry) = self.find_container(map, global_position) {
                let container = container_entry.1;
                return CellBuilder::from_container(container);
            }

            // Otherwise, just draw the tile
            if let Some(tile) = tile_result {
                return CellBuilder::from_tile(&tile);
            }
        }

        // Draw out of range cell
        return CellBuilder::for_blank()
    }
}


impl StatefulWidget for MapWidget {
    type State = Level;

    fn render(mut self, _area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        // TODO (THIS IS VERY SLOW) optimise - we should instead go through the items to render
        // and convert global to local positions for better speed
        // Make need to re-consider the cell builder usage too

        let map_view_area = self.map_view_areas.map_view_area;
        let map_view_area_start = map_view_area.start_position;
        let map_view_area_end = map_view_area.end_position;

        for x in map_view_area_start.x..map_view_area_end.x {
            for y in map_view_area_start.y..map_view_area_end.y {
                // Localise from frame position to a relative co-ordinate e.g [2,2] frame position -> [0, 0]
                let local_position = Position::new(x,y);
                let global_position = self.map_view_areas.local_to_global(local_position).unwrap();

                let position_in_display_area = self.map_view_areas.is_position_in_map_display_area(global_position);
                if position_in_display_area {
                    let mut cell = buf.get_mut(x, y);
                    // Update the cell using the new cell
                    let new_cell = self.build_cell_for_position(_state, global_position,&mut cell);
                    cell.symbol = new_cell.symbol;
                    cell.fg = new_cell.fg;
                    cell.bg = new_cell.bg;
                    cell.modifier = new_cell.modifier;
                }
            }
        }
    }
}
