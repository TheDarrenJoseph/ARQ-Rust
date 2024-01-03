use crate::map::position::{Area, build_rectangular_area, Position};

#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub struct MapViewAreas {
    pub map_area : Area,
    pub map_view_area: Area, // View co-ords
    pub map_display_area : Area  // Map co-ords
}

impl MapViewAreas {
    /*
   Returns true if the given position is inside the range covered by this view
   e.g:
    GIVEN self start_position is x: 5, y: 5 (The map co-ordinate offset)
    AND self view_area is a size of 3 starting from x: 1, y: 1 (this offset is only relevant for display purposed)
    THEN an input Position x 6,7, or 8 would return true (5 + 3 = 8 so range 5-8)
    AND anything above 8 would return false
    AND anything below 5 would return false
*/
    pub(crate) fn is_position_in_map_display_area(&self, position: Position) -> bool {
        self.map_display_area.contains_position(position)
    }

    pub(crate) fn local_to_global(&self, local_position: Position) -> Option<Position> {
        let display_area_start = self.map_display_area.start_position;
        let globalised_x = display_area_start.x + local_position.x;
        let globalised_y = display_area_start.y + local_position.y;
        let global_position = Position::new(globalised_x, globalised_y);
        return Some(global_position);

        None
    }

    pub(crate) fn global_to_local(&self, global_position: Position) -> Option<Position> {
        let map_display_area = self.map_display_area;
        if map_display_area.contains_position(global_position) && self.map_area.contains_position(global_position) {
            let x = global_position.x - map_display_area.start_position.x;
            let y = global_position.y - map_display_area.start_position.y;
            return Some(Position::new(x,y));
        }
        None
    }
}

pub(crate) fn calculate_map_display_area(mut center_position: Position, map_view_area: Area) -> Area {
    let half_of_display_area_x : i32 = (map_view_area.width / 2) as i32;
    let half_of_display_area_y : i32 = (map_view_area.height / 2) as i32;

    // Calculate the display area, centering on the player's position
    let display_area_start = center_position.offset(-half_of_display_area_x, -half_of_display_area_y);

    let display_area_size_x = map_view_area.width - map_view_area.start_position.x;
    let display_area_size_y = map_view_area.height - map_view_area.start_position.y;
    let map_display_area = build_rectangular_area(display_area_start, display_area_size_x, display_area_size_y);

    return map_display_area;
}

#[cfg(test)]
mod tests {
    use crate::map::map_view_areas::{calculate_map_display_area, MapViewAreas};
    use crate::map::position::{Area, Position};
    

    #[test]
    fn test_calculate_map_display_area() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 6
        let PLAYER_POSITION: Position = Position::new(6, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be centered on the player
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_calculate_map_display_area_player_at_y_0() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // AND the player is at position x: 6, y: 0
        let PLAYER_POSITION: Position = Position::new(6, 0);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be centered on the player
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_calculate_map_display_area_player_above_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 5 (1 above the center position)
        let PLAYER_POSITION: Position = Position::new(6, 5);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be unable to follow the player (start position will still be 0,0 as the unsigned value will ensure nothing <0)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_calculate_map_display_area_player_below_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 7 (1 below the center position)
        let PLAYER_POSITION: Position = Position::new(6, 7);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be following the player (start position will now be 0,1 to follow while centered the player)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y + 1, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_calculate_map_display_area_player_left_of_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 5, y: 6 (1 left of the center position)
        let PLAYER_POSITION: Position = Position::new(5, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be unable to follow the player (start position will still be 0,0 as the unsigned value will ensure nothing <0)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_calculate_map_display_area_player_right_of_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 7, y: 6 (1 right of the center position)
        let PLAYER_POSITION: Position = Position::new(7, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be following the player (start position will move to 1,0 as it can follow the player for anything >0)
        assert_eq!(MAP_VIEW_POSITION.x + 1, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.width, map_display_area.width);
        assert_eq!(map_view_area.height, map_display_area.height);
    }

    #[test]
    fn test_is_position_in_map_display_area_valid() {
        // GIVEN a 12x12 map at pos 0,0
        let map_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a view area that covers it all
        let map_view_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a map display area that also covers the whole map
        let map_display_area = Area::new(Position::new(0, 0), 12, 12, );

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to check if a series of positions are in the map display area
        // THEN we expect true to return
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(0,0)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(6,0)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(0,6)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(6,6)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(0, 11)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(11,6)));
        assert!(map_view_areas.is_position_in_map_display_area(Position::new(11,11)));
    }

    #[test]
    fn test_is_position_in_map_display_area_invalid() {
        // GIVEN a 12x12 map at pos 0,0
        let map_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a view area that covers it all
        let map_view_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a map display area that also covers the whole map
        let map_display_area = Area::new(Position::new(0, 0), 12, 12, );

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to check if a series of invalid positions are in the map display area
        // THEN we expect true to return
        assert_eq!(false, map_view_areas.is_position_in_map_display_area(Position::new(12,0)));
        assert_eq!(false, map_view_areas.is_position_in_map_display_area(Position::new(0,12)));
        assert_eq!(false, map_view_areas.is_position_in_map_display_area(Position::new(12,12)));
    }

    #[test]
    fn test_local_to_global() {
        // GIVEN a 12x12 map at pos 0,0 (all maps are currently expected to start at 0,0)
        let map_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a view area that covers it all
        let map_view_area = Area::new(Position::new(0, 0), 12, 12, );
        // AND a map display area that starts at 2,2 and is the same size as the map (So we have 2 tiles of the view beyond the map boundaries)
        let map_display_area = Area::new(Position::new(2, 2), 12, 12, );

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert local position 0,0 to a global map position
        let target_pos = Position::new(0,0);
        let result = map_view_areas.local_to_global(target_pos);
        // THEN we expect it to return as 2,2
        assert_eq!(2, result.unwrap().x);
        assert_eq!(2, result.unwrap().y);
    }

    #[test]
    fn test_local_to_global_minimap_bottom_of_screen_start_of_map() {
        // GIVEN a 12x12 map at pos 0,0 (all maps are currently expected to start at 0,0)
        let map_area = Area::new(Position::new(0, 0), 12, 12 );

        // AND under the assumption that our screen is 80x24 characters
        // a view area is 3x3 in the bottom left of the screen
        let map_view_area = Area::new(Position::new(0, 21), 3, 3 );
        // AND the map display area is trying to display the start of the map
        let map_display_area = Area::new(Position::new(0, 0), 3, 3 );

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert local position 0,0 to a global map position
        let target_pos = Position::new(0,0);
        let result = map_view_areas.local_to_global(target_pos);
        // THEN we expect it to return as 0,0
        assert_eq!(0, result.unwrap().x);
        assert_eq!(0, result.unwrap().y);
    }

    #[test]
    fn test_local_to_global_minimap_bottom_of_screen_end_of_map() {
        // GIVEN a 12x12 map at pos 0,0 (all maps are currently expected to start at 0,0)
        let map_area = Area::new(Position::new(0, 0), 12, 12 );

        // AND under the assumption that our screen is 80x24 characters
        // a view area is 3x3 in the bottom left of the screen
        let map_view_area = Area::new(Position::new(0, 21), 3, 3 );
        // AND the map display area is trying to display the end (bottom right) of the map
        let map_display_area = Area::new(Position::new(9, 9), 3, 3 );

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert local positions to global map positions

        // THEN we expect it to return from 9,9 to 11,11
        let result = map_view_areas.local_to_global(Position::new(0,0));
        assert_eq!(9, result.unwrap().x);
        assert_eq!(9, result.unwrap().y);
        let result = map_view_areas.local_to_global(Position::new(1,1));
        assert_eq!(10, result.unwrap().x);
        assert_eq!(10, result.unwrap().y);
        let result = map_view_areas.local_to_global(Position::new(2,2));
        assert_eq!(11, result.unwrap().x);
        assert_eq!(11, result.unwrap().y);
    }

    #[test]
    fn test_global_to_local_lowest_range() {
        // GIVEN a 6x6 map at pos 0,0 (all maps are currently expected to start at 0,0)
        // We can ignore this for this test
        let map_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a view area of the same size
        // i.e
        // 0,0 0,1 0,2 0,3 0,4 0,5
        // 1,0 1,1 1,2 1,3 1,4 1,5
        // 2,0 2,1 2,2 2,3 2,4 2,5
        // 3,0 3,1 3,2 3,3 3,4 3,5
        // 4,0 4,1 4,2 4,3 4,4 4,5
        // 5,0 5,1 5,2 5,3 5,4 5,5

        let map_view_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a map display area that starts at 3,3 and is the same size as the map
        let map_display_area = Area::new(Position::new(3, 3), 6, 6);
        // Which would then look like so (map_display_area):
        //x ----------->
        //y 3,3 3,4 3,5 N/a N/a N/a
        //| 4,3 4,4 4,5 N/a N/a N/a
        //| 5,4 5,4 5,5 N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        // Which the local (map_view_area) equivalents for are:
        //y 0,0 0,1 0,2 0,3 0,4 0,5
        //| 1,0 1,1 1,2 1,3 1,4 1,5
        //| 2,0 2,1 2,2 2,3 2,4 2,5
        //| 3,0 3,1 3,2 3,3 3,4 3,5
        //| 4,0 4,1 4,2 4,3 4,4 4,5
        //| 5,0 5,1 5,2 5,3 5,4 5,5

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert the global position 3,3 (start of the display area) to a local pos
        let target_pos = Position::new(3,3);
        let result = map_view_areas.global_to_local(target_pos);
        // THEN we expect it to return as 0,0
        assert_eq!(0, result.unwrap().x);
        assert_eq!(0, result.unwrap().y);
    }

    #[test]
    fn test_global_to_local_highest_range() {
        // GIVEN a 6x6 map at pos 0,0 (all maps are currently expected to start at 0,0)
        // We can ignore this for this test
        let map_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a view area of the same size
        // i.e
        // 0,0 0,1 0,2 0,3 0,4 0,5
        // 1,0 1,1 1,2 1,3 1,4 1,5
        // 2,0 2,1 2,2 2,3 2,4 2,5
        // 3,0 3,1 3,2 3,3 3,4 3,5
        // 4,0 4,1 4,2 4,3 4,4 4,5
        // 5,0 5,1 5,2 5,3 5,4 5,5

        let map_view_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a map display area that starts at 3,3 and is the same size as the map
        let map_display_area = Area::new(Position::new(3, 3), 6, 6);
        // Which would then look like so (map_display_area):
        //x ----------->
        //y 3,3 3,4 3,5 N/a N/a N/a
        //| 4,3 4,4 4,5 N/a N/a N/a
        //| 5,4 5,4 5,5 N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        //| N/a N/a N/a N/a N/a N/a
        // Which the local (map_view_area) equivalents for are:
        //y 0,0 0,1 0,2 0,3 0,4 0,5
        //| 1,0 1,1 1,2 1,3 1,4 1,5
        //| 2,0 2,1 2,2 2,3 2,4 2,5
        //| 3,0 3,1 3,2 3,3 3,4 3,5
        //| 4,0 4,1 4,2 4,3 4,4 4,5
        //| 5,0 5,1 5,2 5,3 5,4 5,5

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert the global position 5,5 (largest valid map co-ord in view)
        let target_pos = Position::new(5,5);
        let result = map_view_areas.global_to_local(target_pos);
        // THEN we expect it to return as 2,2
        assert_eq!(2, result.unwrap().x);
        assert_eq!(2, result.unwrap().y);
    }

    #[test]
    fn test_global_to_local_not_in_lower_range() {
        // GIVEN a 6x6 map at pos 0,0 (all maps are currently expected to start at 0,0)
        // We can ignore this for this test
        let map_area = Area::new(Position::new(0, 0), 6, 6);
        let map_view_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a map display area that starts at 3,3 and is the same size as the map
        let map_display_area = Area::new(Position::new(3, 3), 6, 6);
        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert the global position 2,2 (before the map_display_area start) to a local pos
        let target_pos = Position::new(2,2);
        let result = map_view_areas.global_to_local(target_pos);
        // THEN we expect None to return
        assert_eq!(None, result)
    }

    #[test]
    fn test_global_to_local_not_in_upper_range() {
        // GIVEN a 6x6 map at pos 0,0 (all maps are currently expected to start at 0,0)
        // We can ignore this for this test
        let map_area = Area::new(Position::new(0, 0), 6, 6);
        let map_view_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a map display area that starts at 3,3 and is only 3,3
        let map_display_area = Area::new(Position::new(3, 3), 3,3);
        // Which would then look like so (map_display_area):
        //x ----------->
        //y 3,3 3,4 3,5
        //| 4,3 4,4 4,5
        //| 5,4 5,4 5,5

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert the global position 6,6 (outside of the range of the map_display_area)
        let target_pos = Position::new(6,6);
        let result = map_view_areas.global_to_local(target_pos);
        // THEN we expect None to return
        assert_eq!(None, result)
    }

    #[test]
    fn test_global_to_local_not_in_upper_range_overhanging() {
        // GIVEN a 6x6 map at pos 0,0 (all maps are currently expected to start at 0,0)
        // We can ignore this for this test
        let map_area = Area::new(Position::new(0, 0), 6, 6);
        let map_view_area = Area::new(Position::new(0, 0), 6, 6);
        // AND a map display area that starts at 3,3 and is 4,4 in size
        let map_display_area = Area::new(Position::new(3, 3), 4,4);
        // Which would then look like so (map_display_area):
        //x ----------->
        //y 3,3 3,4 3,5 N/a
        //| 4,3 4,4 4,5 N/a
        //| 5,4 5,4 5,5 N/a
        //| N/a N/a N/a N/a

        let map_view_areas = MapViewAreas {
            map_area,
            map_view_area,
            map_display_area
        };

        // WHEN we call to convert the global position 6,6 (outside of the range of the map, but not the map_display_area)
        let target_pos = Position::new(6,6);
        let result = map_view_areas.global_to_local(target_pos);
        // THEN we expect None to return
        assert_eq!(None, result)
    }
}