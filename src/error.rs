use thiserror::Error;

#[derive(Error, Debug)]
pub enum SovaError {
    #[error("Authentication is required.")]
    AuthenticationRequired,
}
