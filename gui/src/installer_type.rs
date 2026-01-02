use crate::message::Message;
use iced::{
    Alignment::Center,
    widget::{container, container::Container, radio, row},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallerType {
    #[default]
    Normal,
    EET,
}

impl InstallerType {
    pub(crate) fn generate_radio(installer_type: Option<Self>) -> Container<'static, Message> {
        let normal = radio(
            "Normal",
            InstallerType::Normal,
            installer_type,
            Message::RadioSelected,
        )
        .size(30)
        .text_size(30);
        let eet = radio(
            "EET",
            InstallerType::EET,
            installer_type,
            Message::RadioSelected,
        )
        .size(30)
        .text_size(30);
        let radio_options = container(row![normal, eet].spacing(30))
            .padding(30)
            .align_x(Center)
            .align_y(Center)
            .style(container::rounded_box);
        radio_options
    }
}
