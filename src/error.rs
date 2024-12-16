use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Already have link: {0}")]
    AlreadyHaveLink(String),
}