use std::collections::HashMap;
use std::slice::Iter;
use tui::layout::Rect;
use crate::map::position::Area;
use crate::ui::ui_areas::UIAreas;
use crate::ui::ui_areas_builder::{UIAreasBuilder};
use crate::ui::ui_layout::LayoutType::{COMBAT_VIEW, SINGLE_MAIN_WINDOW, SINGLE_MAIN_WINDOW_CENTERED, STANDARD_SPLIT};

/*
 * This is essentially a wrapper around UIAreasBuilder that caches the current UIAreas
 */
pub struct UILayout {
    ui_areas : HashMap<LayoutType, UIAreas>,
    ui_areas_builder: UIAreasBuilder
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum LayoutType {
    STANDARD_SPLIT,
    SINGLE_MAIN_WINDOW,
    // N.B this could probably be removed if we add support for centering in all layout types in future
    // This is currently just used for popups/dialogs
    SINGLE_MAIN_WINDOW_CENTERED,
    COMBAT_VIEW
}

impl UILayout {

    fn layout_types() -> Iter<'static, LayoutType> {
        static LAYOUT_TYPES: [LayoutType; 4] = [ STANDARD_SPLIT, SINGLE_MAIN_WINDOW, SINGLE_MAIN_WINDOW_CENTERED, COMBAT_VIEW ];
        LAYOUT_TYPES.iter()
    }

    pub fn new(frame_size: Area) -> UILayout {
        UILayout { ui_areas: HashMap::new(), ui_areas_builder: UIAreasBuilder::new(frame_size) }
    }

    fn rebuild_ui_areas(&mut self, frame_size: Rect, layout_type: LayoutType) -> &UIAreas {
        self.ui_areas_builder.set_frame_size(Area::from_rect(frame_size));
        self.ui_areas_builder.layout_type = layout_type;
        let ui_areas_result = self.ui_areas_builder.build();

        let rebuilt_areas = ui_areas_result.1.clone();
        self.ui_areas.insert(layout_type.clone(), rebuilt_areas);

        return self.get_ui_areas(layout_type);
    }

    /*
        This tries to unwrap and return the current ui_areas.
        ui_areas could be None, so care must be taken when calling this
     */
    pub fn get_ui_areas(&self, layout_type: LayoutType) -> &UIAreas {
        return &self.ui_areas.get(&layout_type).as_ref().unwrap();
    }

    pub fn init_areas(&mut self, frame_size: Rect) {
        for layout_type in Self::layout_types() {
           self.get_or_build_areas(frame_size, *layout_type);
        }
    }

    /*
        This tries to either:
         1. Unwrap and return the current ui_areas
         2. Build a fresh ui_areas using the frame_size provided
         This should always be safe to call as it will build the areas if needed
     */
    pub fn get_or_build_areas(&mut self, frame_size: Rect, layout_type: LayoutType) -> &UIAreas {
        let current_ui_areas = &self.ui_areas.get(&layout_type);
        if current_ui_areas.is_some()  {
            if self.ui_areas_builder.needs_rebuilding(Area::from_rect(frame_size)) {
                return self.rebuild_ui_areas(frame_size, layout_type);
            } else {
                return &self.ui_areas.get(&layout_type).as_ref().unwrap();
            }
        } else {
            return self.rebuild_ui_areas(frame_size, layout_type);
        }
    }
}