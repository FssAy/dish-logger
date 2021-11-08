use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;


#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    InvalidArgument,
    DebuggerAttach,
    OpenProcessFailed,
    PageSizeZero,
    NoToken,
    #[cfg(feature = "async")] AsyncClosed,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            self,
        )
    }
}

impl Deref for Error {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        match self {
            Error::InvalidArgument => &0x01,
            Error::DebuggerAttach => &0x02,
            Error::OpenProcessFailed => &0x03,
            Error::PageSizeZero => &0xF1,
            Error::NoToken => &0x04,
            #[cfg(feature = "async")] Error::AsyncClosed => &0x05,
        }
    }
}

impl std::error::Error for Error { }
