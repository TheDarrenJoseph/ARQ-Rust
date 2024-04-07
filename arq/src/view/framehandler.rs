use tui::layout::Rect;

use crate::map::position::Area;
use crate::ui::ui_areas::UIAreas;

pub mod character_stats;
pub mod character_info;
pub mod console;
pub mod container;
pub mod container_choice;
pub mod util;
pub mod map_generation;
pub mod combat;
pub mod character_equipment;

/*
    FrameHandlers are "dumb" views that simply draw their state (T) or other given input to a terminal frame (the screen)
 */
pub trait FrameHandler<B: tui::backend::Backend, T> {
    /*
        When a FrameHandler "handles" a frame it essentially just draws it's input / content to the frame it is provided a frame by a View/Higher level UI component
     */
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<T>);
}

pub struct FrameData<T> {
    pub data : T,
    // TODO REMOVE this is deprecated in favour of ui_Areas
    // This is for size reference or to restrict the rendering area available
    pub frame_area: Area,
    pub ui_areas: UIAreas
}

impl <T> FrameData<T> {
    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    pub fn get_frame_size(&self) -> Rect {
        self.frame_area.to_rect()
    }

    pub fn get_ui_areas(&self) -> &UIAreas {
        &self.ui_areas
    }
}

impl<B: tui::backend::Backend, T> dyn FrameHandler<B,T> {
}