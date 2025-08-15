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
            let client = Client::builder(run_args.target)
                .port(run_args.port)
                .data(run_args.data)
                .time(run_args.time)
                .chunk_size(run_args.chunk_size)
                .ping_count(run_args.ping_count)
                .return_json(run_args.json)
                .timeout(run_args.timeout)
                .retries(run_args.retries)
                .quiet(run_args.quiet)
                .verbose(run_args.verbose)
                .build()?;

            client.contact()?;
            Ok(())
        }
        Commands::Serve(run_args) => {
            let server = Server::builder()
                .interface(run_args.interface)
                .port(run_args.port)
                .chunk_size(run_args.chunk_size)
                .max_connections(run_args.connections)
                .quiet(run_args.quiet)
                .verbose(run_args.verbose)
                .build()?;

            server.listen()?;
            Ok(())
        }
    }
}
