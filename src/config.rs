use crate::components::menu_bar::{Message as MenuMessage, FileMenu, ViewMenu};
use wave2_wavedb::inout::wave_loader::load_vcd;
use log::info;
use crate::{Message, State};
use iced::Command;

pub fn menu_update(
    app_state: &mut State,
    menu_message: MenuMessage,
) -> Command<Message> {
    match menu_message {
        MenuMessage::File(file_menu) => {
            match file_menu {
                FileMenu::Open => {
                    if app_state.open_file_pending() {
                        Command::none()
                    } else {
                        app_state.set_file_pending(true);
                        Command::perform(load_vcd(), Message::LoadWDB)
                    }
                }
            }
        }
        MenuMessage::View(view_menu) => {
            match view_menu {
                ViewMenu::ImplMe => {
                    info!("Unimplimented menu option! only here to show what multiple tabs look like");
                    Command::none()
                }
            }
        }
    }
}
