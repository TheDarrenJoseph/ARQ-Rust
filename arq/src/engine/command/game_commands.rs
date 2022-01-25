use crate::engine::command::open_container::OpenContainerCommand;

pub struct GameCommands<'a> {
    pub(crate) open : OpenContainerCommand<'a>
}