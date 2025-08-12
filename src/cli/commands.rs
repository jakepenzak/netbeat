use super::{RunArgs, ServeArgs};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a speed test against a target server.
    Run(RunArgs),
    /// Start listening for incoming connections on a target server.
    Serve(ServeArgs),
    // TODO: Install & initialize netbeat on a target server.
    // TODO: Init(InitArgs),
}
