use termion::event::Key;

use crate::map::position::Side;

pub(crate) fn key_to_side(key : Key) -> Option<Side> {
    return match key {
        Key::Down | Key::Char('s') => {
            Some(Side::BOTTOM)
        },
        Key::Up | Key::Char('w')=> {
            Some(Side::TOP)
        },
        Key::Left | Key::Char('a') => {
            Some(Side::LEFT)
        },
        Key::Right | Key::Char('d') => {
            Some(Side::RIGHT)
        },
        _ => {
            None
        }
    }
}