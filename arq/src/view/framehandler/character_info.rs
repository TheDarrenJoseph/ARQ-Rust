/*
    This is responsible for properly displaying the tabbed screen, and each individual tab / frame handler
 */
use crate::view::character_info_view::TabChoice;
use crate::view::framehandler::character_stats::CharacterStatsFrameHandler;
use crate::view::framehandler::container::ContainerFrameHandler;
use crate::view::framehandler::container_choice::ContainerChoiceFrameHandler;

pub struct CharacterInfoFrameHandler {
    pub tab_choice : TabChoice,
    pub container_frame_handlers: Vec<ContainerFrameHandler>,
    pub choice_frame_handler: Option<ContainerChoiceFrameHandler>,
    pub character_view : Option<CharacterStatsFrameHandler>
}
