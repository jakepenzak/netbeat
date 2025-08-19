//! Error utilities for netbeat.

use std::io;
use std::net::AddrParseError;
use thiserror::Error;

/// Enum of errors that can occur in netbeat operations
#[derive(Error, Debug)]
pub enum NetbeatError {
    /// Network connection errors
    #[error("Connection failed: {0}")]
    ConnectionFailedError(#[from] io::Error),

    /// Invalid network address errors
    #[error("Invalid network address: {0}")]
    InvalidAddressError(#[from] AddrParseError),

    /// Protocol errors
    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    /// Timeout errors
    #[error("Operation timed out after {seconds} seconds")]
    TimeoutError { seconds: u64 },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Server errors
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Client errors
    #[error("Client error: {message}")]
    ClientError { message: String },

    /// Test execution errors
    #[error("Test execution error: {message}")]
    TestExecutionError { message: String },

    /// Data parsing errors
    #[error("Parse error: {message}")]
    ParseError { message: String },
}

/// Result type for netbeat operations
pub type Result<T> = std::result::Result<T, NetbeatError>;

impl NetbeatError {
    /// Create a protocol error
    pub fn protocol(message: String) -> Self {
        Self::ProtocolError { message }
    }

    /// Create a timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::TimeoutError { seconds }
    }

    /// Create a configuration error
    pub fn configuration(message: String) -> Self {
        Self::ConfigurationError { message }
    }

    /// Create a server error
    pub fn server(message: String) -> Self {
        Self::ServerError { message }
    }

    /// Create a client error
    pub fn client(message: String) -> Self {
        Self::ClientError { message }
    }

    /// Create a test execution error
    pub fn test_execution(message: String) -> Self {
        Self::TestExecutionError { message }
    }

    /// Create a data parsing error
    pub fn parse(message: String) -> Self {
        Self::ParseError { message }
    }
}
