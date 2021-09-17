use crate::tile::TileDetails;
use crate::position::Position;

pub struct Door {
    pub tile_details : TileDetails,
    position : Position,
    open: bool,
    locked: bool,
    pub health: u16,
    locks: u16,
    unlocked_locks: u16
}

pub trait DoorLike {
    fn open(&mut self);
    fn close(&mut self);
    fn lock(&mut self);
    fn unlock(&mut self);
}

impl DoorLike for Door {
    fn open(&mut self) {
        if !self.locked {
            self.open = true;
        }
    }

    fn close(&mut self) {
        if self.open {
            self.open = false;
        }
    }

    fn lock(&mut self) {
        if !self.locked && !self.open {
            self.locked = true;
        }
    }

    fn unlock(&mut self) {
        if self.locked && !self.open {
            self.locked = false;
        }
    }
}