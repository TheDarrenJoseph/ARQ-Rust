use crate::tile::{Tile, TileDetails};
use crate::position::Position;

pub struct Door {
    pub tile_details : TileDetails,
    pub position : Position,
    pub open: bool,
    pub locked: bool,
    pub health: u16,
    pub locks: u16,
    pub unlocked_locks: u16
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
        if self.locked && !self.open && self.locks > 0  && self.unlocked_locks < self.locks {
            self.locked = false;
        }
    }
}

pub fn build_door(position : Position) -> Door {
    let tile_library = crate::tile::build_library();
    let door_tile_details = tile_library[&Tile::Door].clone();
    Door { tile_details : door_tile_details, position, open: false, locked: false, health: 100, locks: 0, unlocked_locks: 0 }
}

#[cfg(test)]
mod tests {
    use crate::tile::build_library;
    use crate::position::Position;
    use crate::door::{build_door};

    #[test]
    fn test_build_door() {
        let position = Position { x: 1, y: 2};
        let door = build_door(position);

        assert_eq!(1, door.position.x);
        assert_eq!(2, door.position.y);
        assert!(!door.open);
        assert!(!door.locked);
        assert_eq!(100, door.health);
        assert_eq!(0, door.locks);
        assert_eq!(0, door.unlocked_locks);
    }
}