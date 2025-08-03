use anstyle;
use clap::{Args, Parser, Subcommand, builder};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a speed test against a target server.
    Run(RunArgs),
    /// Install & initialize netbeat on a target server.
    // Init(InitArgs),
    /// Start listening for incoming connections on a target server.
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Target server IP address or hostname
    pub target: String,
    /// Target port on server
    #[arg(short, long, default_value_t = 5050)]
    pub port: u16,
    /// Target size of data to be uploaded/downloaded in the speed test including units (eg, 100KB, 10MB, 1GB).
    #[arg(short, long, default_value = "50MB")]
    pub data_size: String,
    /// Packet/chunk size of sent/recieved data including units (eg, 32KB, 64KB, 128KB).
    #[arg(short, long, default_value = "64KB")]
    pub chunk_size: String,
}

// #[derive(Debug, Args)]
// pub struct InitArgs {
//     /// Target server IP address or hostname.
//     pub target: String,
// }

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Target server IP address or hostname
    #[arg(default_value = "0.0.0.0")]
    pub target: String,
    /// Port to listen on.
    #[arg(short, long, default_value_t = 5050)]
    pub port: u16,
    /// Packet/chunk size of received/sent data including units (eg, 32KB, 64KB, 128KB).
    #[arg(short, long, default_value = "64KB")]
    pub chunk_size: String,
}

pub fn get_styles() -> builder::Styles {
    builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Magenta))),
        )
}
