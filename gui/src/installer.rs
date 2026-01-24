use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    widget::{center_x, column, scrollable, text, text_input},
};

use crate::{
    installer_type::InstallerType,
    message::Message,
    state::{eet::Eet, normal::Normal, shared::SharedState},
};

#[derive(Debug)]
pub(crate) enum Installer {
    Normal(Normal),
    Eet(Eet),
}

impl Installer {
    pub(crate) fn new() -> Self {
        Installer::Normal(Normal::new(&SharedState {
            installer_type: Some(InstallerType::Normal),
            ..SharedState::default()
        }))
    }
    pub(crate) fn title(&self) -> String {
        env!("CARGO_PKG_NAME").replace("_", " ")
    }
    pub(crate) fn update(&mut self, message: Message) {
        match self {
            Installer::Normal(state) => match message {
                Message::InstallerPath(input) => {
                    state.shared.installer_bin = input;
                    state.generate_installer_cmd();
                }
                Message::WeiduPath(input) => {
                    state.shared.weidu_bin = input;
                    state.generate_installer_cmd();
                }
                Message::ModDirectory(input) => {
                    state.shared.mod_directory = input;
                    state.generate_installer_cmd();
                }
                Message::RadioSelected(installer_type) => {
                    if InstallerType::Eet == installer_type {
                        state.shared.installer_type = Some(installer_type);
                        *self = Installer::Eet(Eet::new(&state.shared));
                    }
                }
                Message::GameDirectory(input) => {
                    state.game_directory = input;
                    state.generate_installer_cmd();
                }
                Message::WeiduLogPath(input) => {
                    state.weidu_log = input;
                    state.generate_installer_cmd();
                }
                _ => {}
            },
            Installer::Eet(state) => match message {
                Message::InstallerPath(input) => {
                    state.shared.installer_bin = input;
                    state.generate_installer_cmd();
                }
                Message::WeiduPath(input) => {
                    state.shared.weidu_bin = input;
                    state.generate_installer_cmd();
                }
                Message::ModDirectory(input) => {
                    state.shared.mod_directory = input;
                    state.generate_installer_cmd();
                }
                Message::RadioSelected(installer_type) => {
                    if InstallerType::Normal == installer_type {
                        state.shared.installer_type = Some(installer_type);
                        *self = Installer::Normal(Normal::new(&state.shared));
                    }
                }
                Message::BG1GameDirectory(input) => {
                    state.bg1_game_directory = input;
                    state.generate_installer_cmd();
                }
                Message::BG1WeiduLogPath(input) => {
                    state.bg1_weidu_log = input;
                    state.generate_installer_cmd();
                }
                Message::BG2GameDirectory(input) => {
                    state.bg2_game_directory = input;
                    state.generate_installer_cmd();
                }
                Message::BG2WeiduLogPath(input) => {
                    state.bg2_weidu_log = input;
                    state.generate_installer_cmd();
                }
                _ => {}
            },
        }
    }
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let title = text(self.title()).width(Fill).size(100).align_x(Center);
        match self {
            Installer::Normal(Normal {
                shared:
                    SharedState {
                        installer_bin,
                        weidu_bin,
                        mod_directory,
                        installer_type,
                        install_cmd,
                    },
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

                let game_path_input = text_input("Enter path to game directory", game_directory)
                    .on_input(Message::GameDirectory)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let weidu_log_input = text_input("Enter path to weidu log", weidu_log)
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
            Installer::Eet(Eet {
                shared:
                    SharedState {
                        installer_bin,
                        weidu_bin,
                        mod_directory,
                        installer_type,
                        install_cmd,
                    },
                bg1_game_directory,
                bg1_weidu_log,
                bg2_game_directory,
                bg2_weidu_log,
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

                let bg1_game_path_input =
                    text_input("Enter path to bg1 game directory", bg1_game_directory)
                        .on_input(Message::BG1GameDirectory)
                        .padding(15)
                        .size(30)
                        .align_x(Center);

                let bg1_weidu_log_input = text_input("Enter path to bg1 weidu log", bg1_weidu_log)
                    .on_input(Message::BG1WeiduLogPath)
                    .padding(15)
                    .size(30)
                    .align_x(Center);

                let bg2_game_path_input =
                    text_input("Enter path to bg2 game directory", bg2_game_directory)
                        .on_input(Message::BG2GameDirectory)
                        .padding(15)
                        .size(30)
                        .align_x(Center);

                let bg2_weidu_log_input = text_input("Enter path to weidu binary", bg2_weidu_log)
                    .on_input(Message::BG2WeiduLogPath)
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
                    bg1_game_path_input,
                    bg1_weidu_log_input,
                    bg2_game_path_input,
                    bg2_weidu_log_input,
                    cmd
                ]
                .spacing(20);
                scrollable(center_x(content)).into()
            }
        }
    }
}
