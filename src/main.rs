//! A fast, minimal, & lightweight tool for testing network upload and download speeds between a client and server, written entirely in Rust.

use anyhow::Result;
use clap::Parser;
use netbeat::{
    cli::{Cli, Commands},
    core::{Client, Server},
};

fn main() {
    let args = Cli::parse();

    run(args).unwrap_or_else(|err| {
        eprintln!("❌ {err}");
        std::process::exit(1);
    });
}

fn run(args: Cli) -> Result<()> {
    match args.command {
        Commands::Run(run_args) => {
            let client = Client::builder(run_args.target)
                .port(run_args.port)
                .data(run_args.data)
                .time(run_args.time)
                .chunk_size(run_args.chunk_size)?
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
                .chunk_size(run_args.chunk_size)?
                .max_connections(run_args.connections)
                .quiet(run_args.quiet)
                .verbose(run_args.verbose)
                .build()?;

            server.listen()?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_client_server_flow() {
        // Reserve output testing to integration
        let server_args = Cli::parse_from(["netbeat", "serve", "-q"]);

        let _server_handle = thread::spawn(move || {
            run(server_args).expect("Failed to start server");
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(500));

        let client_args = Cli::parse_from(["netbeat", "run", "0.0.0.0", "-t", "1", "-q"]);
        run(client_args).expect("Failed to run client");
    }
}
