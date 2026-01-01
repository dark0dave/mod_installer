use iced::Theme;

use crate::installer::Installer;

mod installer;
mod installer_type;
mod message;
mod state;

fn main() -> iced::Result {
    iced::application(Installer::new, Installer::update, Installer::view)
        .theme(Theme::TokyoNight)
        .title(Installer::title)
        .window_size((500.0, 800.0))
        .run()
}
