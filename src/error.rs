use nix;
use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConnectionError {
  Nix(nix::Error),
  Io(io::Error),
  SocketError(i32),
  SelectError
}

impl From<io::Error> for ConnectionError {
    fn from(err: io::Error) -> ConnectionError {
        ConnectionError::Io(err)
    }
}

impl From<nix::Error> for ConnectionError {
    fn from(err: nix::Error) -> ConnectionError {
        ConnectionError::Nix(err)
    }
}


impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            ConnectionError::Io(ref err) => write!(f, "IO error: {}", err),
            ConnectionError::Nix(ref err) => write!(f, "Nix error: {:?}", err),
            ConnectionError::SelectError => write!(f, "Select error"),
            ConnectionError::SocketError(code) => write!(f, "Socket error: {}", code)
        }
    }
}

impl error::Error for ConnectionError {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            ConnectionError::Io(ref err) => err.description(),
            // Normally we can just write `err.description()`, but the error
            // type has a concrete method called `description`, which conflicts
            // with the trait method. For now, we must explicitly call
            // `description` through the `Error` trait.
            ConnectionError::Nix(_) => "a nix error",
            ConnectionError::SelectError => "select error",
            ConnectionError::SocketError(_) => "socket error"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::NixIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            ConnectionError::Io(ref err) => Some(err),
            ConnectionError::Nix(_) => None,
            ConnectionError::SocketError(_) => None,
            ConnectionError::SelectError => None
        }
    }
}
