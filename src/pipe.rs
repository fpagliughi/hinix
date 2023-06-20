// hinix/src/pipe.rs
//
// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2023, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Pipes.
//!
//! A pipe is aa unidirectional data channel that can be used for
//! interprocess communication. A call to the system pipe() function
//! creates a pipe and returns two separate file handles  - one for
//! the read end of the pipe, and the other for the write end. If the
//! write end of the pipe is closed, any in-progress or subsequent read
//! will return immediately with an EOF (successful read of zero bytes).
//!
//! See:
//! <https://man7.org/linux/man-pages/man2/pipe.2.html>
//!

use crate::Result;
use nix::unistd;
use std::{
    io::{self, Read, Write},
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
};

/// Creates a pipe.
pub fn pipe() -> Result<(WritePipe, ReadPipe)> {
    let (rd_fd, wr_fd) = unistd::pipe()?;
    let rd_pipe = unsafe { ReadPipe::from_raw_fd(rd_fd) };
    let wr_pipe = unsafe { WritePipe::from_raw_fd(wr_fd) };
    Ok((wr_pipe, rd_pipe))
}

/// Read-end of a pipe.
pub struct ReadPipe(OwnedFd);

impl ReadPipe {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(OwnedFd::from_raw_fd(fd))
    }
}

impl Read for ReadPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(unistd::read(self.as_raw_fd(), buf)?)
    }
}

impl AsFd for ReadPipe {
    /// Gets the raw file handle for the read pipe.
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for ReadPipe {
    /// Gets the raw file handle for the read pipe
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// Write-end of a pipe.
pub struct WritePipe(OwnedFd);

impl WritePipe {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(OwnedFd::from_raw_fd(fd))
    }
}

impl Write for WritePipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(unistd::write(self.as_raw_fd(), buf)?)
    }

    fn flush(&mut self) -> io::Result<()> {
        // TODO: Do we need to do anything?
        Ok(())
    }
}

impl AsFd for WritePipe {
    /// Gets the raw file handle for the read pipe.
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for WritePipe {
    /// Gets the raw file handle for the read pipe
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/////////////////////////////////////////////////////////////////////////////
// Unit Tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_pipe() {
        let (mut wr_pipe, mut rd_pipe) = pipe().unwrap();

        thread::spawn(move || {
            wr_pipe.write(&[0x55u8]).unwrap();
        });

        let mut buf = [0u8; 1];
        assert_eq!(1, rd_pipe.read(&mut buf).unwrap());
        assert_eq!(0x55, buf[0]);
    }

    #[test]
    fn test_eof_on_drop() {
        let (wr_pipe, mut rd_pipe) = pipe().unwrap();

        thread::spawn(move || {
            drop(wr_pipe);
        });

        let mut buf = [0u8; 1];
        // Should get an EOF from a read when write-side drops
        assert_eq!(0, rd_pipe.read(&mut buf).unwrap());
    }
}
