use termion::event::Key;

use crate::map::position::Side;

pub(crate) fn key_to_side(key : Key) -> Option<Side> {
    return match key {
        Key::Up | Key::Char('w')=> {
            Some(Side::TOP)
        },
        Key::Down | Key::Char('s') => {
            Some(Side::BOTTOM)
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

pub(crate) fn key_to_input(key : Key) -> Option<Input> {
    return match key {
        Key::Up | Key::Char('w')=> {
            Some(Input::UP(Side::TOP))
        },
        Key::Down | Key::Char('s') => {
            Some(Input::DOWN(Side::BOTTOM))
        },
        Key::Left | Key::Char('a') => {
            Some(Input::LEFT(Side::LEFT))
        },
        Key::Right | Key::Char('d') => {
            Some(Input::RIGHT(Side::RIGHT))
        },
        _ => {
            None
        }
    }
}

pub(crate) fn key_to_action(key : Key) -> Option<Action> {
    return match key {
        Key::Char('i') => {
            Some(Action::ShowInventory)
        },
        Key::Char('l') => {
            Some(Action::LookAround)
        },
        Key::Char('o') => {
            Some(Action::OpenNearby)
        },
        Key::Esc => {
            Some(Action::Escape)
        }
        _ => {
            None
        }
    }
}

/*
  An Action that the Player can take
 */
pub enum Action {
    ShowInventory,
    LookAround,
    OpenNearby,
    Escape // This can open the pause menu, close a container view, etc
}

pub enum Input {
    UP(Side),
    DOWN(Side),
    LEFT(Side),
    RIGHT(Side),
    UNKNOWN
}