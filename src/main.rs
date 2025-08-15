//! A fast, minimal, & lightweight Rust tool for testing network upload and download speeds between a client and server.

use clap::Parser;
use netbeat::{
    cli::{Cli, Commands},
    core::{Client, Server},
};

fn main() {
    let args = Cli::parse();

    run(args).unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        std::process::exit(1);
    });
}

fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        Commands::Run(run_args) => {
            let client = Client::new(
                run_args.target,
                run_args.port.into(),
                run_args.data.as_str().into(),
                run_args.time.into(),
                run_args.chunk_size.as_str().into(),
                run_args.ping_count.into(),
                run_args.json.into(),
                run_args.timeout.into(),
                run_args.retries.into(),
                run_args.quiet.into(),
                run_args.verbose.into(),
            )?;

            client.contact()?;
            Ok(())
        }
        Commands::Serve(run_args) => {
            let server = Server::new(
                run_args.interface.into(),
                run_args.port.into(),
                run_args.chunk_size.into(),
                run_args.connections.into(),
                run_args.quiet.into(),
                run_args.verbose.into(),
            )?;

            server.listen()?;
            Ok(())
        }
    }
}
