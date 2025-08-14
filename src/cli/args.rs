use clap::Args;

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Target server IP address or hostname
    pub target: String,
    /// Target port on server
    #[arg(short, long, default_value_t = 5050)]
    pub port: u16,
    /// Target size of data to be uploaded/downloaded in the speed test including units (eg, 100KiB, 10MiB, 1GiB). If not specified, the test will run until the time limit is reached.
    #[arg(short, long, default_value = "0")]
    pub data: String,
    /// Time for each upload and download test in seconds.
    #[arg(short, long, default_value_t = 15)]
    pub time: u64,
    /// Application buffer size for read/write operations (eg, 32KiB, 64KiB, 128KiB).
    #[arg(short, long, default_value = "64KiB")]
    pub chunk_size: String,
    /// Number of pings to perform for ping test
    #[arg(long, default_value_t = 20)]
    pub ping_count: u32,
    /// Return results as json to stdout
    #[arg(short, long, default_value = "false")]
    pub json: bool,
}

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
