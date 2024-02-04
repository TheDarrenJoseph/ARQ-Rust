use std::collections::HashMap;
use tui::layout::{Constraint, Direction, Layout, Rect};
use crate::map::position::Area;
use crate::ui::ui_areas::{UI_AREA_NAME_CONSOLE, UI_AREA_NAME_MAIN, UIArea, UIAreas};
use crate::ui::ui_util::{center_area, MIN_AREA};

pub struct UIAreasBuilder {
    frame_size: Area,
    layout_type: LayoutType
}

pub enum LayoutType {
    STANDARD_SPLIT,
    SINGLE_MAIN_WINDOW
}

impl UIAreasBuilder {

    pub fn new(frame_size: Area) -> UIAreasBuilder {
        UIAreasBuilder { frame_size, layout_type: LayoutType::STANDARD_SPLIT }
    }



    pub fn needs_rebuilding(&self, frame_size: Rect) -> bool {
        let builder_frame_size = self.frame_size.to_rect();

        // If the currently stored frame area doesn't fit the provided one, we'll need to rebuild it.
        return if builder_frame_size.width > frame_size.width || builder_frame_size.height > frame_size.width {
            true
        } else {
            false
        }
    }

    /*
        Tries to build a single centered Rect based on 80x24 minimum frame size
        If the view is smaller than this, the view will be split as per usual for smaller sizes
     */
    fn get_single_centered_view_area(&self, frame_size: Rect) -> Rect {
        if frame_size.width >= 80 && frame_size.height >= 24 {
            let target = Rect::new(0, 0, 80, 24);
            return center_area(target, frame_size, MIN_AREA).unwrap();
        } else {
            // TODO we probably want to throw an error / show the new error screen for this case
            return frame_size;
        }
    }

    // If the console if visible, splits a frame vertically into the 'main' and lower console areas
    // Otherwise returns the original frame size
    pub fn build(&self) -> UIAreas {
        let frame_size_rect : Rect = self.frame_size.to_rect();
        let rects: Vec<Rect> = match self.layout_type {
            LayoutType::STANDARD_SPLIT => {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(80),
                            Constraint::Percentage(20)
                        ].as_ref()
                    )
                    .split(frame_size_rect)
            },
            LayoutType::SINGLE_MAIN_WINDOW => {
                vec![frame_size_rect]
            }
        };

        let mut areas = HashMap::<String, UIArea>::new();
        areas.insert(UI_AREA_NAME_MAIN.to_string(), UIArea { name: UI_AREA_NAME_MAIN.to_string(), area: rects[0] });
        if rects.len() == 2 {
            areas.insert(UI_AREA_NAME_CONSOLE.to_string(), UIArea { name: UI_AREA_NAME_CONSOLE.to_string(), area: rects[1] });
        }
        UIAreas::new(areas)
    }

    pub fn set_frame_size(&mut self, frame_size: Area) {
        self.frame_size = frame_size;
    }

    pub fn layout_type(&mut self, layout_type: LayoutType) -> &mut UIAreasBuilder {
        self.layout_type = layout_type;
        self
    }
}

#[cfg(test)]
mod tests {
    use tui::layout::Rect;
    use crate::main;
    use crate::map::position::{Area, Position};
    use crate::ui::ui_areas::{UI_AREA_NAME_CONSOLE, UI_AREA_NAME_MAIN, UIArea, UIAreas};
    use crate::ui::ui_areas_builder::{LayoutType, UIAreasBuilder};

    fn assert_area(ui_area: &UIArea, x: u16, y: u16, width: u16, height: u16) {
        assert_eq!(x, ui_area.area.x);
        assert_eq!(y, ui_area.area.y);
        assert_eq!(width, ui_area.area.width);
        assert_eq!(height, ui_area.area.height);
    }

    #[test]
    pub fn test_build_standard_split_80_24() {
        // GIVEN a frame size of 80x24
        let frame_size =  Area::new(Position::new(0,0), 80, 24);
        // WHEN we call to build the standard split areas (default)
        let builder = UIAreasBuilder::new(frame_size);
        let areas = builder.build();
        // THEN we expect 2 areas split 80/30% vertically
        assert_eq!(areas.len(), 2);

        let main_area_result = areas.get_area(UI_AREA_NAME_MAIN);
        assert!(main_area_result.is_some());
        let main_area = main_area_result.unwrap();
        // First area
        // Height is 80% of the 24 lines == 19 (from 0-19 inclusive)
        assert_area(main_area, 0,0, 80, 19);

        let console_area_result = areas.get_area(UI_AREA_NAME_CONSOLE);
        assert!(console_area_result.is_some());
        let console_area = console_area_result.unwrap();
        // Second area
        // Starts at the 2nd of the first area (19) (From 19-24 inclusive)
        // Height is 20% of the 24 lines == 5
        assert_area(console_area, 0,19, 80, 5);
    }

    #[test]
    pub fn test_build_single_main_80_24() {
        // GIVEN a frame size of 80x24
        let frame_size =  Area::new(Position::new(0,0), 80, 24);
        // WHEN we call to build the a single main window
        let areas = UIAreasBuilder::new(frame_size)
            .layout_type(LayoutType::SINGLE_MAIN_WINDOW)
            .build();

        // THEN we expect a single area using 100% of the available space
        assert_eq!(1, areas.len());

        let main_area_result = areas.get_area(UI_AREA_NAME_MAIN);
        assert!(main_area_result.is_some());
        let main_area = main_area_result.unwrap();
        // Height is 80% of the 24 lines == 19 (from 0-19 inclusive)
        assert_area(main_area, 0,0, 80, 24);
    }
}