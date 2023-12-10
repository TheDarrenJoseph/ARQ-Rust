use crate::engine::level::Level;
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::widget::stateful::map_widget::MapWidget;

pub struct MapFrameHandler {
}

pub struct MapFrameHandlerData {
    pub(crate) level: Level,
    pub(crate) map_display_area : Area
}

impl MapFrameHandler {
    pub const fn new() -> MapFrameHandler {
        MapFrameHandler {}
    }

    pub(crate) fn calculate_map_display_area(mut center_position: Position, map_view_area: Area) -> Area {
        let half_of_display_area_x : i32 = (map_view_area.size_x / 2) as i32;
        let half_of_display_area_y : i32 = (map_view_area.size_y / 2) as i32;

        // Calculate the display area, centering on the player's position
        let display_area_start = center_position.offset(-half_of_display_area_x, -half_of_display_area_y);

        let display_area_size_x = map_view_area.size_x - map_view_area.start_position.x;
        let display_area_size_y = map_view_area.size_y - map_view_area.start_position.y;
        let map_display_area = build_rectangular_area(display_area_start, display_area_size_x, display_area_size_y);

        return map_display_area;
    }
}

impl <B : tui::backend::Backend> FrameHandler<B, MapFrameHandlerData> for MapFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<MapFrameHandlerData>) {
        let frame_size = data.frame_size;
        let mut map_widget: MapWidget = MapWidget::new(data.data.level.clone(), data.data.map_display_area.clone());
        frame.render_stateful_widget(map_widget.clone(), frame_size, &mut map_widget);
    }
}

#[cfg(test)]
mod tests {
    use crate::map::position::{Area, Position};
    use crate::view::framehandler::map_framehandler::MapFrameHandler;

    #[test]
    fn test_calculate_map_display_area() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 6
        let PLAYER_POSITION: Position = Position::new(6, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = MapFrameHandler::calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be centered on the player
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.size_x, map_display_area.size_x);
        assert_eq!(map_view_area.size_y, map_display_area.size_y);
    }

    #[test]
    fn test_calculate_map_display_area_player_above_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 5 (1 above the center position)
        let PLAYER_POSITION: Position = Position::new(6, 5);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = MapFrameHandler::calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be unable to follow the player (start position will still be 0,0 as the unsigned value will ensure nothing <0)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.size_x, map_display_area.size_x);
        assert_eq!(map_view_area.size_y, map_display_area.size_y);
    }

    #[test]
    fn test_calculate_map_display_area_player_below_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 6, y: 7 (1 below the center position)
        let PLAYER_POSITION: Position = Position::new(6, 7);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = MapFrameHandler::calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be following the player (start position will now be 0,1 to follow while centered the player)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y + 1, map_display_area.start_position.y);
        assert_eq!(map_view_area.size_x, map_display_area.size_x);
        assert_eq!(map_view_area.size_y, map_display_area.size_y);
    }

    #[test]
    fn test_calculate_map_display_area_player_left_of_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 5, y: 6 (1 left of the center position)
        let PLAYER_POSITION: Position = Position::new(5, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = MapFrameHandler::calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be unable to follow the player (start position will still be 0,0 as the unsigned value will ensure nothing <0)
        assert_eq!(MAP_VIEW_POSITION.x, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.size_x, map_display_area.size_x);
        assert_eq!(map_view_area.size_y, map_display_area.size_y);
    }

    #[test]
    fn test_calculate_map_display_area_player_right_of_center() {
        // GIVEN a map view area of 12x12
        let MAP_VIEW_POSITION: Position = Position::new(0, 0);
        // AND the player is at position x: 7, y: 6 (1 right of the center position)
        let PLAYER_POSITION: Position = Position::new(7, 6);
        let map_view_area = Area::new(MAP_VIEW_POSITION, 12, 12);

        // WHEN we call to build the map display area
        let map_display_area = MapFrameHandler::calculate_map_display_area(PLAYER_POSITION, map_view_area);

        // THEN we expect an area the same size of as map view to return
        // AND it will be following the player (start position will move to 1,0 as it can follow the player for anything >0)
        assert_eq!(MAP_VIEW_POSITION.x + 1, map_display_area.start_position.x);
        assert_eq!(MAP_VIEW_POSITION.y, map_display_area.start_position.y);
        assert_eq!(map_view_area.size_x, map_display_area.size_x);
        assert_eq!(map_view_area.size_y, map_display_area.size_y);
    }
}