//! A fast, minimal, & lightweight Rust tool for testing network upload and download speeds between a client and server.

use clap::Parser;
use netbeat::cli::{Cli, Commands};
use netbeat::core::{Client, Server};

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
                run_args.port,
                run_args.data,
                run_args.time,
                run_args.chunk_size,
                run_args.ping_count,
            )?;
            client.contact()?;
            Ok(())
        }
        Commands::Serve(run_args) => {
            let server = Server::new(run_args.target, run_args.port, run_args.chunk_size)?;

            server.listen()?;
            Ok(())
        }
    }
}
