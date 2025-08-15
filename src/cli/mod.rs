mod args;
mod commands;
mod styles;

use clap::Parser;

pub use args::{RunArgs, ServeArgs};
pub use commands::Commands;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=styles::get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[cfg(test)]
mod tests {
    use crate::core::config::BindInterface;

    use super::*;
    use clap::Parser;

    #[test]
    fn test_run_command_basic() {
        let args = ["netbeat", "run", "192.168.1.1"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Run(run_args) => {
                assert_eq!(run_args.target, "192.168.1.1");
                assert_eq!(run_args.port, 5050); // default
                assert_eq!(run_args.data, "0"); // default
                assert_eq!(run_args.time, 15); // default
                assert_eq!(run_args.chunk_size, "64KiB"); // default
                assert_eq!(run_args.ping_count, 20); // default
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_command_with_all_options() {
        let args = [
            "netbeat",
            "run",
            "example.com",
            "--port",
            "8080",
            "--data",
            "1GiB",
            "--time",
            "30",
            "--chunk-size",
            "128KiB",
            "--ping-count",
            "20",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Run(run_args) => {
                assert_eq!(run_args.target, "example.com");
                assert_eq!(run_args.port, 8080);
                assert_eq!(run_args.data, "1GiB");
                assert_eq!(run_args.time, 30);
                assert_eq!(run_args.chunk_size, "128KiB");
                assert_eq!(run_args.ping_count, 20);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_serve_command_basic() {
        let args = ["netbeat", "serve"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Serve(serve_args) => {
                assert!(matches!(serve_args.interface, BindInterface::All)); // default
                assert_eq!(serve_args.port, 5050); // default
                assert_eq!(serve_args.chunk_size, "64KiB"); // default
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_serve_command_with_options() {
        let args = [
            "netbeat",
            "serve",
            "--interface",
            "all",
            "--port",
            "9090",
            "--chunk-size",
            "32KiB",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Serve(serve_args) => {
                assert!(matches!(serve_args.interface, BindInterface::All));
                assert_eq!(serve_args.port, 9090);
                assert_eq!(serve_args.chunk_size, "32KiB");
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[test]
    fn test_invalid_command_fails() {
        let args = ["netbeat", "invalid-command"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_target_for_run_fails() {
        let args = ["netbeat", "run"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_short_flags() {
        let args = [
            "netbeat",
            "run",
            "localhost",
            "-p",
            "3000",
            "-d",
            "50MiB",
            "-t",
            "60",
            "-c",
            "16KiB",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Run(run_args) => {
                assert_eq!(run_args.target, "localhost");
                assert_eq!(run_args.port, 3000);
                assert_eq!(run_args.data, "50MiB");
                assert_eq!(run_args.time, 60);
                assert_eq!(run_args.chunk_size, "16KiB");
            }
            _ => panic!("Expected Run command"),
        }
    }
}
