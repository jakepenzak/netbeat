//! Core module for netbeat.
//!
//! This module contains the core components of netbeat, including the client, configuration, protocol, and server modules.
//!
//! The **client** module provides the functionality for connecting to a netbeat server and sending/receiving data.
//!
//! The **config** module provides the default parametrization for client and server modules.
//!
//! The **protocol** module provides the custom protocol for network communication over netbeat client and server.
//!
//! The **server** module provides the functionality for running a netbeat server and handling incoming connections.

pub mod client;
pub mod config;
pub mod protocol;
pub mod server;

pub use client::Client;
pub use server::Server;
