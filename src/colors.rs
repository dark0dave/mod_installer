use anstyle::{
    AnsiColor::{Blue, BrightCyan, BrightRed, Green, Red, Yellow},
    Color::Ansi,
    Style,
};
use clap::builder::Styles;

pub fn styles() -> Styles {
    Styles::styled()
        .usage(Style::new().bold().underline().fg_color(Some(Ansi(Yellow))))
        .header(Style::new().bold().underline().fg_color(Some(Ansi(Blue))))
        .literal(Style::new().fg_color(Some(Ansi(Green))))
        .invalid(Style::new().bold().fg_color(Some(Ansi(Red))))
        .error(Style::new().bold().fg_color(Some(Ansi(BrightRed))))
        .valid(Style::new().bold().underline().fg_color(Some(Ansi(Green))))
        .placeholder(Style::new().fg_color(Some(Ansi(BrightCyan))))
}
