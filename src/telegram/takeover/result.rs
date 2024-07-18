//! Replika errors
#![allow(unused)]

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error("Credits spent out, total: {0}")]
    Credits(u8),
}

impl Error {
    /// If the error is credits
    pub fn is_credits(&self) -> bool {
        matches!(self, Error::Credits(_))
    }
}
