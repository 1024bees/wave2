use crate::components::menu_bar::Message as MenuMessage;
use wave2_wavedb::inout::wave_loader::load_vcd;

use crate::{Message, State};
use iced::Command;

pub fn menu_update(
    app_state: &mut State,
    menu_message: MenuMessage,
) -> Command<Message> {
    match menu_message {
        MenuMessage::OpenFile => {
            if app_state.open_file_pending() {
                Command::none()
            } else {
                app_state.set_file_pending(true);
                Command::perform(load_vcd(), Message::LoadWDB)
            }
        }
    }
}
