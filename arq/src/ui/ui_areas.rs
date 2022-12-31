use tui::layout::Rect;

pub struct UIAreas {
    main_area : Rect,
    console_area: Option<Rect>
}

impl UIAreas {

    pub const fn new(main_area : Rect, console_area: Option<Rect>) -> UIAreas {
        UIAreas { main_area, console_area }
    }

    pub fn get_main_area(&self) -> Rect {
        self.main_area
    }

    pub fn set_main_area(&mut self, main_area: Rect) {
        self.main_area = main_area;
    }

    pub fn get_console_area(&self) -> Option<Rect> {
        self.console_area
    }

    pub fn set_console_area(&mut self, console_area: Option<Rect>) {
        self.console_area = console_area;
    }
}


