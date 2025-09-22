use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("I/O Error")]
    IoError(#[from] std::io::Error),
    #[error("Slint platform error")]
    SlintFailure(#[from] slint::PlatformError),
    #[error("Unknown application error")]
    Unknown(String),
}
