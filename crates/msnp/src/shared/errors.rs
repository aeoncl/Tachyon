use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentifierError {
    #[error("Could not validate email address: {}", .0)]
    InvalidEmailAddress(String)
}