use std::io::Error;

use crate::engine::engine_helpers::input_handler::handle_input;
use crate::engine::game_engine::GameEngine;
use crate::ui::ui::get_input_key;
use crate::view::game_over_view::GameOverChoice;

pub async fn game_loop<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>) -> Result<Option<GameOverChoice>, Error> {
    let game_over_choice = player_turn(engine).await?;
    npc_turns(engine)?;
    return Ok(game_over_choice);
}

async fn player_turn<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>)  -> Result<Option<GameOverChoice>, Error> {
    let key = get_input_key()?;
    return Ok(handle_input(engine, key).await?);
}

fn npc_turns<B: tui::backend::Backend + Send>(_engine: &mut GameEngine<B>)  -> Result<(), Error> {
    // TODO NPC movement
    return Ok(());
}
