//! Error utilities for netbeat.

use std::io;
use thiserror::Error;

/// Enum of errors that can occur in netbeat operations
#[derive(Error, Debug)]
pub enum NetbeatError {
    /// Network connection errors
    #[error("Connection error: {0}")]
    ConnectionError(#[from] io::Error),

    /// Protocol errors
    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    /// Server errors
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Client errors
    #[error("Client error: {message}")]
    ClientError { message: String },

    /// Test execution errors
    #[error("Test execution error: {message}")]
    TestExecutionError { message: String },
}

/// Result type for netbeat operations
pub type Result<T> = std::result::Result<T, NetbeatError>;

impl NetbeatError {
    /// Create a protocol error
    pub fn protocol(message: String) -> Self {
        Self::ProtocolError { message }
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
}
