use termion::event::Key;

use crate::map::position::Side;

pub(crate) fn key_to_side(key : Key) -> Option<Side> {
    return match key {
        Key::Down => {
            Some(Side::BOTTOM)
        },
        Key::Up => {
            Some(Side::TOP)
        },
        Key::Left => {
            Some(Side::LEFT)
        },
        Key::Right => {
            Some(Side::RIGHT)
        },
        _ => {
            None
        }
    }
}