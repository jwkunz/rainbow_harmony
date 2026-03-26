use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum RainbowError {
    Usage(String),
    Audio(String),
}

impl RainbowError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn audio(message: impl Into<String>) -> Self {
        Self::Audio(message.into())
    }
}

impl Display for RainbowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Audio(message) => f.write_str(message),
        }
    }
}

impl Error for RainbowError {}
