use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error;
use crate::error_utils::error_result;
use crate::map::objects::container::{Container, ContainerType, wrap_item};
use crate::map::objects::items::Item;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EquipmentSlot {
    HEAD,
    TORSO,
    LEGS,
    FEET,
    PRIMARY,
    SECONDARY,
}

impl Display for EquipmentSlot {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug)]
pub struct Equipment {
    slots : HashMap<EquipmentSlot, Item>
}

impl Equipment {
    pub fn is_equipped(&self, slot: EquipmentSlot) -> bool {
        self.slots.get(&slot).is_some()
    }

    pub fn equip(&mut self, container_item : Container, slot: EquipmentSlot) -> Result<(), Error> {
        if !self.is_equipped(slot.clone()) {
            return match container_item.get_container_type() {
                // Only objects can be equipped
                ContainerType::OBJECT => {
                    let item = container_item.get_self_item().clone();
                    self.slots.insert(slot, item);
                    Ok(())
                },
                ct => {
                    error_result(format!("Unsupported container_type: {}", ct))
                }
            }
        } else {
            return error_result(format!("Cannot equip. Equipment slot: {} is already taken.", slot))
        }
    }

    pub fn unequip(&mut self, slot: EquipmentSlot) -> Result<Container, Error> {
        if self.is_equipped(slot.clone()) {
            let item = self.slots.remove(&slot).unwrap();
            let container = wrap_item(item);
            return Ok(container);
        } else {
            return error_result(format!("Cannot unequip. Equipment slot: {} is empty.", slot))
        }
    }
}