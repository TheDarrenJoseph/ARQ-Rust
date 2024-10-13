use crate::engine::engine_helpers::input_handler::handle_input;
use crate::engine::game_engine::GameEngine;
use crate::error::errors::{ErrorType, ErrorWrapper};
use crate::ui::ui::get_input_key;
use crate::view::game_over_view::GameOverChoice;

pub async fn game_loop<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>) -> Result<Option<GameOverChoice>, ErrorWrapper> {
    let game_over_result = player_turn(engine).await;
    match game_over_result {
        Ok(goc) => {
            npc_turns(engine)?;
            return Ok(goc);
        },
        Err(e) => {
            match e.error_type {
                // Handle internal errors by putting the message into the console
                ErrorType::INTERNAL => {
                    engine.ui_wrapper.ui.set_console_buffer(e.message.clone().unwrap());
                    engine.ui_wrapper.re_render()?;
                    // TODO use a mockable input handler
                    
                    //self.input_handler.get_input_key()?;
                    return Ok(None)
                }
                ErrorType::IO => {
                    return Err(e)
                }
            }
        }
    }
}

async fn player_turn<B: tui::backend::Backend + Send>(engine: &mut GameEngine<B>)  -> Result<Option<GameOverChoice>, ErrorWrapper> {
    let key = get_input_key()?;
    return Ok(handle_input(engine, key).await?);
}

fn npc_turns<B: tui::backend::Backend + Send>(_engine: &mut GameEngine<B>)  -> Result<(), ErrorWrapper> {
    // TODO NPC movement
    return Ok(());
}
