//! A fast, lightweight Rust tool for testing network upload and download speeds between a client and server.

mod cli;
mod client;
mod conf;
mod server;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use client::contact;
use conf::NetbeatConf;
use server::listen;

fn main() {
    let args = Cli::parse();

    run(args).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    });
}

fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        Commands::Run(run_args) => {
            let client_conf = NetbeatConf::client(
                run_args.target,
                run_args.port,
                run_args.data_size,
                run_args.chunk_size,
            )?;
            contact(client_conf)?;
            Ok(())
        }
        Commands::Serve(run_args) => {
            let server_conf =
                NetbeatConf::server(run_args.target, run_args.port, run_args.chunk_size)?;

            listen(server_conf)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
