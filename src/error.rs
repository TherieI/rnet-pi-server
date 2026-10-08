use openrnet::error::RnetSockErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChairError {
    #[error("User interrupted the chair operation")]
    UserInterrupt,
    #[error("RNET socket Failure")]
    Socket(#[from] RnetSockErr)
}
