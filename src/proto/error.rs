use crate::codec::{RecvError, SendError};
use crate::frame::Reason;

use std::fmt;
use std::io;
use std::sync::Arc;

/// Either an H2 reason  or an I/O error
#[derive(Clone, Debug)]
pub enum Error {
    Reset(Reason, Initiator),
    Io(Arc<io::Error>),
}

#[derive(Clone, Copy, Debug)]
pub enum Initiator {
    User,
    Library,
    Remote,
}

impl Error {
    pub(crate) fn is_local(&self) -> bool {
        match *self {
            Self::Reset(_, initiator) => initiator.is_local(),
            Self::Io(_) => true,
        }
    }

    pub(crate) fn user_go_away(reason: Reason) -> Self {
        Self::Reset(reason, Initiator::User)
    }

    pub(crate) fn library_go_away(reason: Reason) -> Self {
        Self::Reset(reason, Initiator::Library)
    }

    pub(crate) fn remote_go_away(reason: Reason) -> Self {
        Self::Reset(reason, Initiator::Remote)
    }
}

impl Initiator {
    fn is_local(&self) -> bool {
        match *self {
            Self::User | Self::Library => true,
            Self::Remote => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reset(reason, _) => reason.fmt(fmt),
            Self::Io(ref error) => error.fmt(fmt),
        }
    }
}

impl fmt::Display for Initiator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Self::User => "user",
            Self::Library => "library",
            Self::Remote => "remote",
        })
    }
}

impl From<io::ErrorKind> for Error {
    fn from(src: io::ErrorKind) -> Self {
        Error::Io(Arc::new(src.into()))
    }
}

impl From<io::Error> for Error {
    fn from(src: io::Error) -> Self {
        Error::Io(Arc::new(src))
    }
}

impl From<Error> for RecvError {
    fn from(src: Error) -> Self {
        Self::Connection(src)
    }
}

impl From<Error> for SendError {
    fn from(src: Error) -> Self {
        Self::Connection(src)
    }
}
