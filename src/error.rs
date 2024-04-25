use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorWrapper {
    #[error("Generic error")]
    GenericError,
}
