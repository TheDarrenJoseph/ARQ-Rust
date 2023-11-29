use crate::character::battle::Battle;
use crate::character::equipment::WeaponSlot;
use crate::view::combat_view::{CombatCallbackData, CombatResult, CombatView};
use crate::view::util::callback::CallbackHandler;

#[derive(Clone)]
pub enum CombatTurnChoice {
    ATTACK(WeaponSlot),
    FLEE
}

pub struct Combat {
    pub(crate) battle: Battle
}

impl CallbackHandler<CombatCallbackData> for Combat {
    fn handle_callback(&mut self, data: CombatCallbackData) -> Option<CombatCallbackData> {
        let mut result_data: CombatCallbackData = data.clone();
        let mut messages : Vec<String> = Vec::new();
        match data.choice {
            CombatTurnChoice::ATTACK(_) => {
                messages.push(String::from("You attempt attack..."));
            }
            CombatTurnChoice::FLEE => {
                messages.push(String::from("You attempt to run away..."));
            }
        }
        result_data.result = Some(CombatResult { messages });
        Some(result_data)
    }
}