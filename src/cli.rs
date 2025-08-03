use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a speed test against a target server.
    Run(RunArgs),
    /// Install & initialize netbeat on a target server.
    Init(InitArgs),
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
    /// Size of data to be sent in the speed test including units (100KB, 10MB, 1GB).
    #[arg(short, long, default_value = "5MB")]
    pub data_size: String,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Target server IP address or hostname.
    pub target: String,
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Target server IP address or hostname
    #[arg(default_value = "0.0.0.0")]
    pub target: String,
    /// Port to listen on.
    #[arg(short, long, default_value_t = 5050)]
    pub port: u16,
    /// Buffer size on the server including units (100KB, 10MB, 1GB).
    #[arg(short, long, default_value = "16KB")]
    pub buffer_size: String,
}
