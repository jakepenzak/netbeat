#![doc = include_str!("../README.md")]

pub mod cli;
pub mod core;
pub mod output;
pub mod utils;

pub use core::config::BindInterface;
pub use core::{Client, Server};
pub use output::reports::{NetbeatReport, PingReport, SpeedReport};
pub use utils::error::{NetbeatError, Result};
