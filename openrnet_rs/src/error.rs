use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RnetSockErr {
    #[error("Failed to initialize socket: {0}")]
    Create(#[source] io::Error),
    #[error("Failed to read from the socket")]
    Read(#[from] io::Error),
    #[error("Socket operation timed out")]
    TimedOut,
    #[error("Sockcan internal error")]
    Internal(#[from] socketcan::Error),
}
