use crate::engine::level::Level;
use crate::map::map_view_areas::MapViewAreas;
use crate::map::position::{Area, build_rectangular_area, Position};
use crate::view::framehandler::{FrameData, FrameHandler};
use crate::widget::stateful::map_widget::MapWidget;

pub struct MapFrameHandler {
}

#[derive(Clone)]
#[derive(Debug)]
pub struct MapFrameHandlerData {
    pub(crate) level: Level,
    pub map_view_areas: MapViewAreas
}

impl MapFrameHandler {
    pub const fn new() -> MapFrameHandler {
        MapFrameHandler {}
    }


}

impl <B : tui::backend::Backend> FrameHandler<B, MapFrameHandlerData> for MapFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, mut data: FrameData<MapFrameHandlerData>) {
        let frame_size = data.frame_size;
        let mut map_widget: MapWidget = MapWidget::new( data.data.map_view_areas );
        frame.render_stateful_widget(map_widget.clone(), frame_size, &mut data.data.level);
    }
}

#[cfg(test)]
mod tests {
    use crate::map::position::{Area, Position};
    use crate::view::framehandler::map_framehandler::MapFrameHandler;

}