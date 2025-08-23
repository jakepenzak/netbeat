//! Command-line interface styles for netbeat.

use anstyle::{AnsiColor, Color, Style};
use clap::builder;

/// Command-line interface styles for netbeat.
pub fn get_styles() -> builder::Styles {
    builder::Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Magenta))))
}
