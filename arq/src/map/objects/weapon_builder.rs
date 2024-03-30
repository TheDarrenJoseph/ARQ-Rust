use uuid::Uuid;

use crate::character::equipment::EquipmentSlot;
use crate::error::errors::GenericError;
use crate::map::objects::items::{Dimensions, Item, ItemForm, ItemType, MaterialType, Weapon};
use crate::map::tile::{Colour, Symbol};

pub struct WeaponBlueprint {
    weapon: Weapon,
    item_type: ItemType,
    item_form: ItemForm,
    material_type: MaterialType,
    name : String,
    symbol : Symbol, // TODO allow using the colour for 'special' weapons?
    weight : f32,
    value : i32,
    equipment_slot: Option<EquipmentSlot>
}

impl WeaponBlueprint {
    /* Abstract damage amount */
    fn determine_damage(material_type: MaterialType) -> Result<i32, GenericError>  {
        return match material_type {
            MaterialType::IRON => {
                Ok(20)
            },
            MaterialType::STEEL => {
                Ok(30)
            },
            MaterialType::SILVER => {
                Ok(18)
            },
            MaterialType::GOLD => {
                Ok(10)
            },
            // Future TODO potentially allow UNKNOWN types, but we'd need to provide data for it's properties as a weapon
            _ => {
                Err(GenericError::new(format!("Unsupported material type for a weapon: {:?}", material_type)))
            }
        }
    }

    pub fn new(material_type: MaterialType, item_form: ItemForm) -> Result<WeaponBlueprint, GenericError> {
        let material_strength = Self::determine_damage(material_type.clone())?;
        let density_grams_cm3 = material_type.density_grams_cm3() as f32;

        let weight_kg: f32 = match item_form.clone() {
            ItemForm::BLADED(sword_type) => {
                let dimensions_cm = sword_type.dimensions_cm();
                let area_cm3 = dimensions_cm.area();
                (density_grams_cm3 * area_cm3) / 1000.0
            },
            _ => {
                // Default to 1KG
                1.0
            }
        };

        let name = match item_form.clone() {
            ItemForm::BLADED(sword_type) => {
                 format!("{} {}", &material_type.name(), sword_type.name())
            }
            _ => {
                "".to_string() // Default to empty name
            }
        };

        Ok(WeaponBlueprint {
            weapon: Weapon {
                damage: material_strength
            },
            item_type: ItemType::ITEM,
            item_form,
            material_type,
            name,
            symbol: Symbol { character: '|', colour: Colour::White},
            weight: weight_kg,
            value: 1,
            equipment_slot: None
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BladedWeaponType {
    DAGGER,
    ARMING,
    LONG
}

impl BladedWeaponType {
    pub fn name(&self) -> String {
        return match self {
            BladedWeaponType::DAGGER => {
                String::from("Dagger")
            },
            BladedWeaponType::ARMING => {
                 String::from("Arming Sword")
            },
            BladedWeaponType::LONG => {
                 String::from("Longsword")
            }
        }
    }

    pub fn dimensions_cm(&self) -> Dimensions {
        return match self {
            BladedWeaponType::DAGGER => {
                Dimensions {
                    height: 0.4, // ~4mm blade thickness
                    width: 2.4, // ~24mm blade width
                    length: 43.0,
                }
            },
            BladedWeaponType::ARMING => {
                Dimensions {
                    height: 0.4, // ~4mm blade thickness
                    width: 4.5, // ~45mm blade width
                    length: 97.0, // ~97cm total length
                }
            },
            BladedWeaponType::LONG => {
                Dimensions {
                    height: 0.4, // ~4mm blade thickness
                    width: 4.5, // ~45mm blade width
                    length: 110.0, // 110cm total length
                }
            }
        }
    }
}

pub struct WeaponBuilder {
    blueprint: WeaponBlueprint
}

impl WeaponBuilder {
    pub fn new(blueprint : WeaponBlueprint) -> WeaponBuilder {
        WeaponBuilder { blueprint }
    }

    pub fn build(&self) -> Item {
        let blueprint = &self.blueprint;
        Item::weapon(Uuid::new_v4(), blueprint.name.clone(), ItemForm::OTHER(blueprint.name.clone()), blueprint.material_type.clone(), blueprint.symbol.character.clone(), blueprint.weight.clone(), blueprint.value.clone(), blueprint.weapon.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::map::objects::items::{ItemForm, ItemType, MaterialType, Weapon};
    use crate::map::objects::weapon_builder::{BladedWeaponType, WeaponBlueprint, WeaponBuilder};

    #[test]
    pub fn test_build_steel_dagger() {
        // GIVEN a builder with the relevant blueprint for a Steel Dagger
        let blueprint = WeaponBlueprint::new(MaterialType::STEEL, ItemForm::BLADED(BladedWeaponType::DAGGER)).unwrap();
        let builder = WeaponBuilder::new(blueprint);

        // WHEN we call to build the item
        let weapon = builder.build();

        assert_eq!(ItemType::WEAPON(Weapon { damage: 30}), weapon.item_type);
        assert_eq!("Steel Dagger", weapon.get_name());
        // Weight in Kilograms
        assert_eq!(0.33024, weapon.weight);
    }

    #[test]
    pub fn test_build_steel_arming_sword() {
        // GIVEN a builder with the relevant blueprint for a Steel Arming Sword
        let blueprint = WeaponBlueprint::new(MaterialType::STEEL, ItemForm::BLADED(BladedWeaponType::ARMING)).unwrap();
        let builder = WeaponBuilder::new(blueprint);

        // WHEN we call to build the item
        let weapon = builder.build();

        assert_eq!(ItemType::WEAPON(Weapon { damage: 30}), weapon.item_type);
        assert_eq!("Steel Arming Sword", weapon.get_name());
        // Weight in Kilograms
        assert_eq!(1.3968, weapon.weight);
    }

    #[test]
    pub fn test_build_steel_long_sword() {
        // GIVEN a builder with the relevant blueprint for a Steel Long Sword
        let blueprint = WeaponBlueprint::new(MaterialType::STEEL, ItemForm::BLADED(BladedWeaponType::LONG)).unwrap();
        let builder = WeaponBuilder::new(blueprint);

        // WHEN we call to build the item
        let weapon = builder.build();

        assert_eq!(ItemType::WEAPON(Weapon { damage: 30}), weapon.item_type);
        assert_eq!("Steel Longsword", weapon.get_name());
        // Weight in Kilograms
        assert_eq!(1.5840001, weapon.weight);
    }
}