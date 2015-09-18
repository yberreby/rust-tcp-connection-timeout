extern crate nix;
use std::net::TcpStream;
use std::time::Duration;
use std::os::unix::io::FromRawFd;

mod error;
pub use error::ConnectionError;

pub fn tcp_connect_with_timeout(socket_addr: std::net::SocketAddr, timeout: Duration) -> Result<TcpStream, ConnectionError> {
  // Create a socket file descriptor.
  let socket_fd = try!(nix::sys::socket::socket(
    nix::sys::socket::AddressFamily::Inet,
    nix::sys::socket::SockType::Stream,
    nix::sys::socket::SockFlag::empty()
  ));

  // Set the socket to non-blocking mode so we can `select()` on it.
  try!(nix::fcntl::fcntl(
    socket_fd,
    nix::fcntl::FcntlArg::F_SETFL(nix::fcntl::O_NONBLOCK)
  ));

  let connection_result = nix::sys::socket::connect(
    socket_fd,
    &(nix::sys::socket::SockAddr::Inet(nix::sys::socket::InetAddr::from_std(&socket_addr)))
  );

  match connection_result {
    Ok(_) => (),
    Err(e) => {
      match e {
        nix::Error::Sys(errno) => {
          match errno {
            nix::errno::Errno::EINPROGRESS => (), // socket is non-blocking so an EINPROGRESS is to be expected
            _ => return Err(ConnectionError::from(e))
          }
        }
        nix::Error::InvalidPath => unreachable!() //
      }
    }
  }

  let mut timeout_timeval = nix::sys::time::TimeVal {
    tv_sec: timeout.as_secs() as i64,
    tv_usec: timeout.subsec_nanos() as i32
  };

  // Create a new fd_set monitoring our socket file descriptor.
  let mut fdset = nix::sys::select::FdSet::new();
  fdset.insert(socket_fd);

  // `select()` on it, will return when the connection succeeds or times out.
  let select_res = try!(nix::sys::select::select(
    socket_fd + 1,
    None,
    Some(&mut fdset),
    None,
    &mut timeout_timeval
  ));

  // This it what fails if `addr` is unreachable.
  if select_res != 1 {
    println!("select return value: {}", select_res);
    return Err(ConnectionError::SelectError);
  }

  // Make sure the socket encountered no error.
  let socket_error_code = try!(nix::sys::socket::getsockopt(
    socket_fd,
    nix::sys::socket::sockopt::SocketError
  ));

  if socket_error_code != 0 {
    return Err(ConnectionError::SocketError(socket_error_code));
  }

  // Set the socket back to blocking mode so it can be used with std's I/O facilities.
  try!(nix::fcntl::fcntl(
    socket_fd,
    nix::fcntl::FcntlArg::F_SETFL(nix::fcntl::OFlag::empty())
  ));

  // Wrap it in a TcpStream and return that stream.
  Ok(
    unsafe { TcpStream::from_raw_fd(socket_fd) }
  )
}
