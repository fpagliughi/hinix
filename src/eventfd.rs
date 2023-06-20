// hinix/src/eventfd.rs
//
// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2018-2020, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Linux event (eventfd) objects.
//!
//! See:
//! <https://man7.org/linux/man-pages/man2/eventfd.2.html>
//!

use crate::{Error, Result};
use nix::{self, sys::eventfd, unistd};
use std::{
    mem,
    os::{
        raw::c_uint,
        unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
    },
    slice,
};

/// The size, in bytes, of the value held by an eventfd.
/// This is the required size of a buffer that is used for reads and writes,
/// as the value is a u64.
const EFD_VAL_SIZE: usize = mem::size_of::<u64>();

/// The flags used to create an EventFd
pub type EfdFlags = eventfd::EfdFlags;

/// An event object that can be used as a wait/notify mechanism between
/// user-space applications, threads in an app, or between the kernel and
/// user-space.
///
/// This is a simpler, more efficient signaling mechanism than a pipe, if
/// event notification is all that is required by the application.
///
/// The event is seen as a normal file handle, and thus can be used in
/// combination with other handles such as from sockets, pipes, etc,
/// in a poll/epoll/select call to provide additional signaling
/// capabilities.
#[derive(Debug)]
pub struct EventFd(OwnedFd);

impl EventFd {
    /// Create a new event object.
    ///
    /// This is the default configuration of the event object with no flags.
    /// When read, the value is returned and the count is reset to zero.
    ///
    /// # Parameters
    ///
    /// `initval` The initial value held by the object
    pub fn new(initval: u64) -> Result<EventFd> {
        Self::with_flags(initval, EfdFlags::empty())
    }

    /// Create a new event object with the semaphore option.
    ///
    /// This is applies the EDF_SEMAPHORE flag. When read, the value
    /// returned is 1, and the value is decremented by 1.
    ///
    /// # Parameters
    ///
    /// `initval` The initial value held by the object
    pub fn new_semaphore(initval: u64) -> Result<EventFd> {
        Self::with_flags(initval, EfdFlags::EFD_SEMAPHORE)
    }

    /// Create a new event object with the specified flags.
    ///
    /// # Parameters
    /// `initval` The initial value held by the object
    /// `flags` The flags used to create the object
    ///
    /// <http://man7.org/linux/man-pages/man2/eventfd.2.html>
    pub fn with_flags(initval: u64, flags: EfdFlags) -> Result<EventFd> {
        let fd = eventfd::eventfd(initval as c_uint, flags)?;
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        Ok(EventFd(fd))
    }

    /// Try to clone the event object by making a dup() of the OS file handle.
    pub fn try_clone(&self) -> Result<Self> {
        let fd = self
            .0
            .try_clone()
            .map_err(|e| Error::try_from(e).unwrap_or_else(|_| Error::from_i32(0)))?;
        Ok(EventFd(fd))
    }

    /// Reads the value of the event object.
    pub fn read(&self) -> Result<u64> {
        let mut buf: [u8; 8] = [0; EFD_VAL_SIZE];
        if unistd::read(self.0.as_raw_fd(), &mut buf)? != EFD_VAL_SIZE {
            return Err(Error::EIO);
        }
        let val: u64 = unsafe { *(&buf as *const u8 as *const u64) };
        Ok(val)
    }

    /// Writes a value to the event object.
    ///
    /// # Parameters
    /// `val` The value to _add_ to the one held by the object.
    pub fn write(&self, val: u64) -> Result<()> {
        let buf = unsafe { slice::from_raw_parts(&val as *const u64 as *const u8, EFD_VAL_SIZE) };
        if unistd::write(self.0.as_raw_fd(), buf)? != EFD_VAL_SIZE {
            return Err(Error::EIO);
        }
        Ok(())
    }
}

impl AsFd for EventFd {
    /// Gets the raw file handle for the event object.
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for EventFd {
    /// Gets the raw file handle for the event object.
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/////////////////////////////////////////////////////////////////////////////
// Unit Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal() {
        let evtfd = EventFd::new(0).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // Writing a value should get us the same back on a read.
        evtfd.write(1).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // Try another value that's not '1'
        evtfd.write(42).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(42, n);

        // Multiple writes should sunm the value
        evtfd.write(5).unwrap();
        evtfd.write(6).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(11, n);
    }

    #[test]
    fn test_non_blocking() {
        let evtfd = EventFd::with_flags(0, EfdFlags::EFD_NONBLOCK).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // No value in object should get us an EAGAIN error.
        match evtfd.read() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(Error::EAGAIN, err),
        }

        // Writing a value should get us the same back on a read.
        evtfd.write(6).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(6, n);

        // The read should have cleared the value, so another is an error.
        match evtfd.read() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(Error::EAGAIN, err),
        }

        // Try another value that's not '1'
        evtfd.write(42).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(42, n);
    }

    #[test]
    fn test_semaphore() {
        let evtfd = EventFd::new_semaphore(0).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // Signal then read back should get us a 1.
        evtfd.write(1).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // Try another value that's not 1.
        evtfd.write(2).unwrap();

        // Each read should return 1.
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        let n = evtfd.read().unwrap();
        assert_eq!(1, n);
    }

    #[test]
    fn test_semaphore_non_blocking() {
        let evtfd =
            EventFd::with_flags(0, EfdFlags::EFD_SEMAPHORE | EfdFlags::EFD_NONBLOCK).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // Try another value that's not '1'
        evtfd.write(2).unwrap();

        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // The read should have cleared the value, so another is an error.
        match evtfd.read() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(Error::EAGAIN, err),
        }
    }
}
