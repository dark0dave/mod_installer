use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    widget::{center_x, column, scrollable, text, text_input},
};

use crate::{installer_type::InstallerType, message::Message, state::State};

#[derive(Debug)]
pub(crate) enum Installer {
    Init(State),
}

impl Installer {
    pub(crate) fn new() -> Self {
        Installer::Init(State::new())
    }
    pub(crate) fn title(&self) -> String {
        env!("CARGO_PKG_NAME").replace("_", " ")
    }
    pub(crate) fn update(&mut self, message: Message) {
        match self {
            Installer::Init(state) => match message {
                Message::InstallerPath(input) => {
                    state.installer_bin = input;
                    state.generate_installer_cmd();
                }
                Message::WeiduPath(input) => {
                    state.weidu_bin = input;
                    state.generate_installer_cmd();
                }
                Message::ModDirectory(input) => {
                    state.mod_directory = input;
                    state.generate_installer_cmd();
                }
                Message::RadioSelected(installer_type) => {
                    state.installer_type = Some(installer_type);
                    state.generate_installer_cmd();
                }
                Message::GameDirectory(input) => {
                    state.game_directory = input;
                    state.generate_installer_cmd();
                }
                Message::WeiduLogPath(input) => {
                    state.weidu_log = input;
                    state.generate_installer_cmd();
                }
            },
        }
    }
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let title = text(self.title()).width(Fill).size(100).align_x(Center);
        match self {
            Installer::Init(State {
                installer_bin,
                weidu_bin,
                mod_directory,
                installer_type,
                install_cmd,
                game_directory,
                weidu_log,
                ..
            }) => {
                let installer_path = text_input("Enter path to installer binary", installer_bin)
                    .on_input(Message::InstallerPath)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let weidu_path = text_input("Enter path to weidu binary", weidu_bin)
                    .on_input(Message::WeiduPath)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let mod_directory_input = text_input("Enter Mods directory", mod_directory)
                    .on_input(Message::ModDirectory)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let radio_install_type = InstallerType::generate_radio(*installer_type);

                let game_path_input = text_input("Enter path to weidu binary", game_directory)
                    .on_input(Message::GameDirectory)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let weidu_log_input = text_input("Enter path to weidu binary", weidu_log)
                    .on_input(Message::WeiduLogPath)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let cmd = text_input(install_cmd, install_cmd)
                    .width(Fill)
                    .align_x(Center)
                    .size(30);

                let content = column![
                    title,
                    installer_path,
                    weidu_path,
                    mod_directory_input,
                    radio_install_type,
                    game_path_input,
                    weidu_log_input,
                    cmd
                ]
                .spacing(20);
                scrollable(center_x(content)).into()
            }
        }
    }
}
