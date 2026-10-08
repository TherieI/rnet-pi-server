use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChairError {
    #[error("User interrupted the chair operation")]
    UserInterrupt,
}
