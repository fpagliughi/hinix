// hinix/src/eventfd.rs

//! Module to manage Linux event objects.
use std::os::unix::io::{RawFd, AsRawFd};

use std::{slice, mem};
use libc::{c_uint};
use nix;
use nix::unistd;
use nix::sys::eventfd;
use nix::errno::Errno;
use Result;

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
pub struct EventFd {
    fd: RawFd,
}

impl EventFd {
    /// Create a new event object.
    ///
    /// # Parameters
    /// `initval` The initial value held by the object
    /// `flags` The flags used to create the object
    ///
    /// http://man7.org/linux/man-pages/man2/eventfd.2.html
    pub fn new(initval: c_uint, flags: EfdFlags) -> Result<EventFd> {
        let fd = eventfd::eventfd(initval, flags)?;
        Ok(EventFd { fd })
    }

    /// Reads the value of the event object
    pub fn read(&self) -> Result<u64> {
        let mut buf: [u8; 8] = [0; EFD_VAL_SIZE];
        if unistd::read(self.fd, &mut buf)? != EFD_VAL_SIZE {
            // TODO: Whet Errno to use?
            return Err(nix::Error::Sys(Errno::EIO));
        }
        let val: u64 = unsafe {
            *(&buf as *const u8 as *const u64)
        };
        Ok(val)
    }

    /// Writes a value to the event object.
    ///
    /// # Parameters
    /// `val` The value to _add_ to the one held by the object.
    pub fn write(&self, val: u64) -> Result<()> {
        let buf = unsafe {
            slice::from_raw_parts(&val as *const u64 as *const u8,
                                  EFD_VAL_SIZE)
        };
        if unistd::write(self.fd, &buf)? != EFD_VAL_SIZE {
            // TODO: What Errno to use?
            return Err(nix::Error::Sys(Errno::EIO));
        }
        Ok(())
    }
}

impl AsRawFd for EventFd {
    /// Gets the raw file handle for the event object.
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        let _ = unistd::close(self.fd);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::io::AsRawFd;
    use nix::Error;

    #[test]
    fn test_normal() {
        let evtfd = EventFd::new(0, EfdFlags::empty()).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // Writing a value should get us the same back on a read.
        evtfd.write(1).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // Try another value that's not '1'
        evtfd.write(42).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(42, n);
    }

    #[test]
    fn test_non_blocking() {
        let evtfd = EventFd::new(0, EfdFlags::EFD_NONBLOCK).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // No value in object should get us an EAGAIN error.
        match evtfd.read() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(Error::Sys(Errno::EAGAIN), err),
        }

        // Writing a value should get us the same back on a read.
        evtfd.write(1).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // The read should have cleared the value, so another is an error.
        match evtfd.read() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(Error::Sys(Errno::EAGAIN), err),
        }

        // Try another value that's not '1'
        evtfd.write(42).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(42, n);
    }

    #[test]
    fn test_semaphore() {
        let evtfd = EventFd::new(0, EfdFlags::EFD_SEMAPHORE).unwrap();
        assert!(evtfd.as_raw_fd() >= 0);

        // Writing a value should get us the same back on a read.
        evtfd.write(1).unwrap();
        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        // Try another value that's not '1'
        evtfd.write(2).unwrap();

        let n = evtfd.read().unwrap();
        assert_eq!(1, n);

        let n = evtfd.read().unwrap();
        assert_eq!(1, n);
    }

    #[test]
    fn test_semaphore_non_blocking() {
        let evtfd = EventFd::new(0, EfdFlags::EFD_SEMAPHORE | EfdFlags::EFD_NONBLOCK).unwrap();
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
            Err(err) => assert_eq!(Error::Sys(Errno::EAGAIN), err),
        }

    }
}


