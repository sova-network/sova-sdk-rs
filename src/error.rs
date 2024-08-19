use thiserror::Error;

#[derive(Error, Debug)]
pub enum MevtonError {
    #[error("Authentication is required.")]
    AuthenticationRequired,
}
