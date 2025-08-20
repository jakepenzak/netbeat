//! # Netbeat
//!
//! A fast, minimal, & lightweight Rust tool for testing network upload and download speeds between a client and server.
//!
//! Netbeat provides both a command-line interface and a library for measuring network performance,
//! monitoring connectivity, and analyzing network behavior in your applications.
//!
//! ## Command Line Interface
//!
//! ### Basic Usage
//!
//! Show help:
//! ```text
//! $ netbeat --help
//! A fast, minimal, & lightweight Rust tool for testing network upload and download speeds between a client and server
//!
//! Usage: netbeat <COMMAND>
//!
//! Commands:
//!   run    Run a speed test against a target server
//!   serve  Start listening for incoming connections on a target server
//!   help   Print this message or the help of the given subcommand(s)
//!
//! Options:
//!   -h, --help     Print help
//!   -V, --version  Print version
//! ```
//!
//! ### Running Speed Tests
//!
//! Run a basic speed test:
//! ```text
//! $ netbeat run example.com
//! 🔗 Connecting to example.com:5050...
//! ✅ Connected successfully
//! 📤 Testing upload speed...
//! 📥 Testing download speed...
//! 📊 Results:
//!   Upload:   125.4 Mbps
//!   Download: 89.7 Mbps
//!   Latency:  23ms
//! ```
//!
//! Run with custom parameters:
//! ```text
//! $ netbeat run example.com --port 8080 --data 1GiB --time 30
//! 🔗 Connecting to example.com:8080...
//! ✅ Connected successfully
//! 📤 Testing upload speed (1GiB target)...
//! [████████████████████████████████] 100% | 1.0 GiB | 30s
//! 📥 Testing download speed (1GiB target)...
//! [████████████████████████████████] 100% | 1.0 GiB | 30s
//! 📊 Results:
//!   Upload:   156.8 Mbps (1.0 GiB in 28.4s)
//!   Download: 143.2 Mbps (1.0 GiB in 31.1s)
//!   Latency:  15ms (avg of 20 pings)
//! ```
//!
//! Get JSON output for scripting:
//! ```text
//! $ netbeat run example.com --json
//! {
//!   "target": "example.com",
//!   "port": 5050,
//!   "upload_mbps": 125.4,
//!   "download_mbps": 89.7,
//!   "latency_ms": 23,
//!   "timestamp": "2024-01-15T10:30:45Z"
//! }
//! ```
//!
//! ### Starting a Server
//!
//! Start server on all interfaces:
//! ```text
//! $ netbeat serve
//! 🚀 Starting netbeat server...
//! 📡 Listening on 0.0.0.0:5050
//! ✅ Server ready for connections
//!
//! [2024-01-15 10:30:45] 📥 Connection from 192.168.1.100:52341
//! [2024-01-15 10:30:46] 📊 Speed test completed: 156.8 Mbps upload, 143.2 Mbps download
//! ```
//!
//! Start server with custom settings:
//! ```text
//! $ netbeat serve --interface localhost --port 9090 --connections 10
//! 🚀 Starting netbeat server...
//! 📡 Listening on 127.0.0.1:9090 (max 10 connections)
//! ✅ Server ready for connections
//! ```
//!
//! ### Run Command Options
//!
//! ```text
//! $ netbeat run --help
//! Run a speed test against a target server
//!
//! Usage: netbeat run [OPTIONS] <TARGET>
//!
//! Arguments:
//!   <TARGET>  Target server IP address or hostname
//!
//! Options:
//!   -p, --port <PORT>              Target port on server (1-65535) [default: 5050]
//!   -t, --time <TIME>              Time limit per test direction in seconds (1-3600) [default: 15]
//!   -d, --data <DATA>              Target size of data to be uploaded/downloaded [default: 0]
//!   -c, --chunk-size <CHUNK_SIZE>  Buffer size for read/write operations [default: 64KiB]
//!       --ping-count <PING_COUNT>  Number of pings to perform for ping test (1-1000) [default: 20]
//!   -j, --json                     Return results as json to stdout
//!       --timeout <TIMEOUT>        Connection timeout in seconds [default: 10]
//!       --retries <RETRIES>        Number of retry attempts on connection failure [default: 3]
//!   -q, --quiet                    Suppress progress output (results & errors only)
//!   -v, --verbose                  Enable verbose output
//!   -h, --help                     Print help
//! ```
//!
//! ## Library Usage
//!
//! ### Basic Client Usage
//!
//! ```text
//! use netbeat::{Client, Result};
//!
//! fn main() -> Result<()> {
//!     let client = Client::builder("example.com")
//!         .port(5050)
//!         .data("1GiB".to_string())
//!         .time(30)
//!         .build()?;
//!
//!     let result = client.contact()?;
//!     println!("Test completed successfully");
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Server Setup
//!
//! ```text
//! use netbeat::{Server, Result, core::config::BindInterface};
//!
//! fn main() -> Result<()> {
//!     let server = Server::builder()
//!         .interface(BindInterface::All)
//!         .port(5050)
//!         .max_connections(100)
//!         .build()?;
//!
//!     server.listen()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **🚀 Fast**: Optimized for high-performance network testing
//! - **🔧 Configurable**: Extensive options for customizing tests
//! - **📊 Detailed Metrics**: Upload/download speeds, latency, and more
//! - **🌐 Cross-platform**: Works on Linux, macOS, and Windows
//! - **📝 JSON Output**: Perfect for automation and scripting
//! - **🔒 Robust**: Comprehensive error handling and retry logic

pub mod cli;
pub mod core;
pub mod output;
pub mod utils;

pub use core::{Client, Server};
pub use utils::error::{NetbeatError, Result};
