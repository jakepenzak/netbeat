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
    // Install & initialize netbeat on a target server.
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
    /// Target size of data to be uploaded/downloaded in the speed test including units (eg, 100KiB, 10MiB, 1GiB).
    #[arg(short, long, default_value = "100MiB")]
    pub data: String,
    /// Time for each upload and download test in seconds. Overrides --data. 0 defaults to --data.
    #[arg(short, long, default_value = "0")]
    pub time: u64,
    /// Application buffer size for read/write operations (eg, 32KiB, 64KiB, 128KiB).
    #[arg(short, long, default_value = "64KiB")]
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
    /// Packet/chunk size of received/sent data including units (eg, 32KiB, 64KiB, 128KiB).
    #[arg(short, long, default_value = "64KiB")]
    pub chunk_size: String,
}

pub fn get_styles() -> builder::Styles {
    builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))),
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

#[cfg(test)]
mod tests {
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
                assert_eq!(run_args.data, "100MiB"); // default
                assert_eq!(run_args.time, 1); // default
                assert_eq!(run_args.chunk_size, "64KiB"); // default
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
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Run(run_args) => {
                assert_eq!(run_args.target, "example.com");
                assert_eq!(run_args.port, 8080);
                assert_eq!(run_args.data, "1GiB");
                assert_eq!(run_args.time, 30);
                assert_eq!(run_args.chunk_size, "128KiB");
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
                assert_eq!(serve_args.target, "0.0.0.0"); // default
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
            "127.0.0.1",
            "--port",
            "9090",
            "--chunk-size",
            "32KiB",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Serve(serve_args) => {
                assert_eq!(serve_args.target, "127.0.0.1");
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
