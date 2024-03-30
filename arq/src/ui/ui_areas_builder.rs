use std::collections::HashMap;

use log::info;
use tui::layout::{Constraint, Direction, Layout, Rect};

use crate::map::position::{Area, Position};
use crate::ui::ui_areas::{BorderedArea, UI_AREA_NAME_CONSOLE, UI_AREA_NAME_MAIN, UI_AREA_NAME_MINIMAP, UIArea, UIAreas};
use crate::ui::ui_layout::LayoutType;
use crate::ui::ui_layout::LayoutType::{CombatView, SingleMainWindow, SingleMainWindowCentered, StandardSplit};
use crate::ui::ui_util::center_area;
use crate::view::MIN_RESOLUTION;

pub struct UIAreasBuilder {
    frame_size: Area,
    pub(crate) layout_type: LayoutType
}

/*
    Tries to build a single centered Rect based on 80x24 minimum frame size
    If the view is smaller than this, the view will be split as per usual for smaller sizes
 */
fn build_single_main_window_centered_areas(total_area: Area) -> HashMap::<String, UIArea> {
    let mut areas = HashMap::<String, UIArea>::new();
    let target = Rect::new(0, 0, MIN_RESOLUTION.width, MIN_RESOLUTION.height);
    let area = center_area(target, total_area.to_rect(), MIN_RESOLUTION).unwrap();
    areas.insert(UI_AREA_NAME_MAIN.to_string(), UIArea { name: UI_AREA_NAME_MAIN.to_string(), area });
    areas
}

pub(crate) fn build_single_main_window_areas(total_area: Area) -> HashMap::<String, UIArea>   {
    let mut areas = HashMap::<String, UIArea>::new();
    areas.insert(UI_AREA_NAME_MAIN.to_string(), UIArea { name: UI_AREA_NAME_MAIN.to_string(), area: total_area });
    areas
}

fn build_split_areas(total_area: Area) -> HashMap::<String, UIArea> {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Percentage(20)
            ].as_ref()
        )
        .split(total_area.to_rect());

    let mut areas = HashMap::<String, UIArea>::new();
    let main_area = Area::from_rect(rects[0]);
    let console_area = Area::from_rect(rects[1]);
    areas.insert(UI_AREA_NAME_MAIN.to_string(), UIArea { name: UI_AREA_NAME_MAIN.to_string(), area: main_area });
    areas.insert(UI_AREA_NAME_CONSOLE.to_string(), UIArea { name: UI_AREA_NAME_CONSOLE.to_string(), area: console_area });
    areas
}

/*
 The combat view uses a special UI layout consisting of:
 1. The 'main area' which is the top 80% vertical of the screen, this is the same as normal except it's split into left/right columns
 2. The 'console area' which is an area below the main area with left-offset to allow for the minimap
 2. The 'minimap area' which is an area to the left of the console area

pub main_area : BorderedArea,
pub console_area : BorderedArea,
pub minimap_area : BorderedArea
 */
fn build_combat_view_areas(total_area: Area) -> HashMap::<String, UIArea>  {
    // Split the entire view area into main and console
    let ui_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Percentage(20)
            ].as_ref()
        )
        .split(total_area.to_rect());

    let mut console_rect = ui_areas[1].clone();

    // To make space for the minimap, remove some of the left-hand side of the console area
    let frame_width_float = total_area.width as f32;
    // Give 20% of the frame width to the minimap area
    let minimap_width = (frame_width_float / 100.0 * 20.0) as u16;
    // Leaving 80% for the console
    let console_width = (frame_width_float / 100.0 * 80.0) as u16;
    console_rect.x = minimap_width;
    console_rect.width = console_width;

    let minimap_position = Position::new(console_rect.x - minimap_width, console_rect.y);
    let minimap_ui_area = Area::new(minimap_position, minimap_width, console_rect.height);

    let main_area = BorderedArea::from_area(Area::from_rect(ui_areas[0])).unwrap();
    let console_area = BorderedArea::from_rect(console_rect).unwrap();
    let minimap_area = BorderedArea::from_area(minimap_ui_area).unwrap();

    let mut areas = HashMap::<String, UIArea>::new();
    areas.insert(UI_AREA_NAME_MAIN.to_string(), UIArea { name: UI_AREA_NAME_MAIN.to_string(), area: main_area.outer });
    areas.insert(UI_AREA_NAME_CONSOLE.to_string(), UIArea { name: UI_AREA_NAME_CONSOLE.to_string(), area: console_area.outer });
    areas.insert(UI_AREA_NAME_MINIMAP.to_string(), UIArea { name: UI_AREA_NAME_MINIMAP.to_string(), area: minimap_area.outer });

    areas
}

impl UIAreasBuilder {

    pub fn new(frame_size: Area) -> UIAreasBuilder {
        UIAreasBuilder { frame_size, layout_type: LayoutType::StandardSplit }
    }

    pub fn needs_rebuilding(&self, total_area: Area) -> bool {
        let builder_frame_size = self.frame_size.to_rect();

        // If the currently stored frame area doesn't fit the provided one, we'll need to rebuild it.
        return if builder_frame_size.width > total_area.width || builder_frame_size.height > total_area.height {
            true
        } else {
            false
        }
    }

    /*
      Builds the replacement for Frame.size() based on the target resolution
      This is meant to represent the total available view area
     */
    fn build_total_area(&self) -> Area {
        let frame_size : Rect = self.frame_size.to_rect();
        if frame_size.width >= 80 && frame_size.height >= 24 {
            return Area::new(Position::zero(), frame_size.width, frame_size.height);
        } else {
            // TODO we probably want to throw an error / show the new error screen for this case
            panic!("Screen resolution not supported, cannot build UI areas..");
        }
    }

    // If the console if visible, splits a frame vertically into the 'main' and lower console areas
    // Otherwise returns the original frame size
    pub fn build(&self) -> (LayoutType, UIAreas) {
        let total_area = self.build_total_area();
        info!("Building layout of type: {:?} with total area: {:?}", self.layout_type, total_area);
        match self.layout_type {
            StandardSplit => {
                let areas = build_split_areas(total_area);
                return (StandardSplit, UIAreas::new(areas))
            },
            SingleMainWindow => {
                let areas= build_single_main_window_areas(total_area);
                return (SingleMainWindow, UIAreas::new(areas))
            },
            SingleMainWindowCentered => {
                let areas = build_single_main_window_centered_areas(total_area);
                return (SingleMainWindowCentered, UIAreas::new(areas))
            },
            CombatView => {
                let areas = build_combat_view_areas(total_area);
                return (CombatView, UIAreas::new(areas))
            }
        };
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
    use crate::map::position::{Area, Position};
    use crate::ui::ui_areas::{UI_AREA_NAME_CONSOLE, UI_AREA_NAME_MAIN, UI_AREA_NAME_MINIMAP, UIArea};
    use crate::ui::ui_areas_builder::UIAreasBuilder;
    use crate::ui::ui_layout::LayoutType;

    fn assert_area(ui_area: &UIArea, x: u16, y: u16, width: u16, height: u16) {
        assert_eq!(x, ui_area.area.start_position.x);
        assert_eq!(y, ui_area.area.start_position.y);
        assert_eq!(width, ui_area.area.width);
        assert_eq!(height, ui_area.area.height);
    }

    #[test]
    pub fn test_build_standard_split_80_24() {
        // GIVEN a frame size of 80x24
        let frame_size =  Area::new(Position::new(0,0), 80, 24);
        // WHEN we call to build the standard split areas (default)
        let builder = UIAreasBuilder::new(frame_size);
        let result = builder.build();
        let areas = result.1;

        // THEN we expect 2 areas split 80/30% vertically
        assert_eq!(LayoutType::StandardSplit, result.0);
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
        let result = UIAreasBuilder::new(frame_size)
            .layout_type(LayoutType::SingleMainWindow)
            .build();

        let areas = result.1;

        // THEN we expect a single area using 100% of the available space
        assert_eq!(LayoutType::SingleMainWindow, result.0);
        assert_eq!(1, areas.len());

        let main_area_result = areas.get_area(UI_AREA_NAME_MAIN);
        assert!(main_area_result.is_some());
        let main_area = main_area_result.unwrap();
        // Height is 80% of the 24 lines == 19 (from 0-19 inclusive)
        assert_area(main_area, 0,0, 80, 24);
    }

    #[test]
    fn test_build_combat_view_areas() {
        // GIVEN a frame size of 80x24
        let frame_size =  Area::new(Position::new(0,0), 80, 24);
        // WHEN we call to build view areas
        let result = UIAreasBuilder::new(frame_size)
            .layout_type(LayoutType::CombatView)
            .build().1;

        // THEN we expect
        // A main area of
        // the entire width
        // with 80% of the total height
        let main_area = result.get_bordered_area(UI_AREA_NAME_MAIN);
        assert_eq!(80, main_area.outer.width);
        // With 80% of the 24 lines (24/100 * 80 = 19.2, rounded down = 19)
        assert_eq!(19, main_area.outer.height);

        // A console area of y
        let console_area = result.get_bordered_area(UI_AREA_NAME_CONSOLE);
        // With 80% of the total frame width (80/100 * 80 = 64)
        assert_eq!(64, console_area.outer.width);
        // With 20% of the total height of 24 lines (24/100 * 20 = 4.8, rounded up = 5)
        assert_eq!(5, console_area.outer.height);

        // A minimap area
        let minimap_area = result.get_bordered_area(UI_AREA_NAME_MINIMAP);
        // With 20% of the total frame width (80/100 * 30 = 16)
        assert_eq!(16, minimap_area.outer.width);
        // AND the same height as the console width
        assert_eq!(5, minimap_area.outer.height);

    }
}