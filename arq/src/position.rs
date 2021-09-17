#[derive(Copy, Clone)]
pub struct Position {
    pub x : u16,
    pub y : u16
}

#[derive(Copy, Clone)]
pub struct Area {
    pub start_position : Position,
    pub end_position : Position,
    pub size : u16
}

#[derive(Copy, Clone)]
pub enum Side {
    LEFT,
    RIGHT,
    TOP,
    BOTTOM
}

#[derive(Copy, Clone)]
pub struct AreaSide {
    pub area: Area,
    pub side : Side
}

pub fn all_sides() -> [Side; 4] {
    [Side::LEFT,Side::RIGHT,Side::TOP,Side::BOTTOM]
}

pub fn build_line(start_position : Position, size: u16, side: Side) -> AreaSide {
    let start_x = start_position.x;
    let start_y = start_position.y;

    let start_position;
    let end_position;
    match side {
        Side::LEFT => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x, y: start_y + size};
        },
        Side::RIGHT => {
            start_position = Position { x : start_x + size, y: start_y};
            end_position = Position { x : start_x, y: start_y + size};
        },
        Side::TOP => {
            start_position = Position { x : start_x, y: start_y};
            end_position = Position { x : start_x + size, y: start_y};
        },
        Side::BOTTOM => {
            start_position = Position { x : start_x, y: start_y + size};
            end_position = Position { x : start_x + size, y: start_y + size};
        }
    }
    let area = Area { start_position, end_position, size };
    AreaSide { area, side }
}

pub fn build_area(start_position : Position, size: u16) -> Area {
    let start_x = start_position.x;
    let start_y = start_position.y;
    let end_position = Position { x : start_x + size, y: start_y + size};
    Area { start_position, end_position, size }
}