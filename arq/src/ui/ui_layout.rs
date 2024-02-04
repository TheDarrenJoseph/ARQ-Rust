use tui::layout::Rect;
use crate::map::position::Area;
use crate::ui::ui_areas::UIAreas;
use crate::ui::ui_areas_builder::LayoutType::STANDARD_SPLIT;
use crate::ui::ui_areas_builder::UIAreasBuilder;

/*
 * This is essentially a wrapper around UIAreasBuilder that caches the current UIAreas
 */
pub struct UILayout {
    ui_areas : Option<UIAreas>,
    ui_areas_builder: UIAreasBuilder
}

impl UILayout {
    pub fn new(frame_size: Area) -> UILayout {
        UILayout { ui_areas: None, ui_areas_builder: UIAreasBuilder::new(frame_size) }
    }

    fn rebuild_ui_areas(&mut self, frame_size: Rect) -> &UIAreas {
        self.ui_areas_builder.set_frame_size(Area::from_rect(frame_size));
        let ui_areas = self.ui_areas_builder.build();
        self.ui_areas = Some(ui_areas.clone());
        return &self.ui_areas.as_ref().unwrap();
    }

    /*
        This tries to unwrap and return the current ui_areas.
        ui_areas could be None, so care must be taken when calling this
     */
    pub fn get_ui_areas(&self) -> &UIAreas {
        return self.ui_areas.as_ref().unwrap();
    }

    /*
        This tries to either:
         1. Unwrap and return the current ui_areas
         2. Build a fresh ui_areas using the frame_size provided
         This should always be safe to call as it will build the areas if needed
     */
    pub fn get_or_build_areas(&mut self, frame_size: Rect) -> &UIAreas {
        let current_ui_areas = &self.ui_areas;
        if current_ui_areas.is_some()  {
            if self.ui_areas_builder.needs_rebuilding(frame_size) {
                return self.rebuild_ui_areas(frame_size);
            } else {
                return self.ui_areas.as_ref().unwrap();
            }
        } else {
            return self.rebuild_ui_areas(frame_size);
        }
    }
}