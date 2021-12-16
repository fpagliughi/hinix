// hinix/src/msgqueue.rs
//
// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2021, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Module to manage Posix Message Queues
//!
//! See:
//! <https://man7.org/linux/man-pages/man7/mq_overview.7.html>
//!

use nix::{
    self,
    sys::stat::Mode,
    mqueue::{self, MQ_OFlag, mq_attr_member_t},
};
use std::{
    os::unix::io::{AsRawFd, RawFd},
};
use libc::mqd_t;
use std::ffi::CString;
use crate::Result;

/// The default priority for the Message Queue send operation.
pub const DEFAULT_PRIO: u32 = 0;

// Note the MqAttr struct is copypasta from the *nix crate.
// This is necessary since the *nix version doesn't provide access to the
// components other than the flags, so it's impossible to get the other
// parameters for an existing queue.

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MqAttr {
    mq_attr: libc::mq_attr,
}

impl MqAttr {
    pub fn new(
        mq_flags: mq_attr_member_t,
        mq_maxmsg: mq_attr_member_t,
        mq_msgsize: mq_attr_member_t,
        mq_curmsgs: mq_attr_member_t
    ) -> MqAttr
    {
        use std::mem;
        let mut attr = mem::MaybeUninit::<libc::mq_attr>::uninit();
        unsafe {
            let p = attr.as_mut_ptr();
            (*p).mq_flags = mq_flags;
            (*p).mq_maxmsg = mq_maxmsg;
            (*p).mq_msgsize = mq_msgsize;
            (*p).mq_curmsgs = mq_curmsgs;
            MqAttr { mq_attr: attr.assume_init() }
        }
    }

    pub const fn flags(&self) -> mq_attr_member_t {
        self.mq_attr.mq_flags
    }

    pub const fn max_msg(&self) -> mq_attr_member_t {
        self.mq_attr.mq_maxmsg
    }

    pub const fn msg_size(&self) -> mq_attr_member_t {
        self.mq_attr.mq_msgsize
    }

    pub const fn current_msg(&self) -> mq_attr_member_t {
        self.mq_attr.mq_curmsgs
    }
}

impl From<MqAttr> for mqueue::MqAttr {
    fn from(attr: MqAttr) -> Self {
        mqueue::MqAttr::new(
            attr.flags(),
            attr.max_msg(),
            attr.msg_size(),
            attr.current_msg()
        )
    }
}

/// Get message queue attributes
fn mq_getattr(mqd: mqd_t) -> Result<MqAttr> {
    use std::mem;
    use nix::errno::Errno;

    let mut attr = mem::MaybeUninit::<libc::mq_attr>::uninit();
    let res = unsafe { libc::mq_getattr(mqd, attr.as_mut_ptr()) };
    Errno::result(res).map(|_| unsafe{MqAttr { mq_attr: attr.assume_init() }})
}

// -----


/// A Posix Message Queue
#[derive(Debug)]
pub struct MsgQueue {
    /// The OS file descriptor
    mq: mqd_t,
    /// Max number of messages
    max_msg: usize,
    /// The size of each message
    msg_size: usize,
}

impl MsgQueue {
    /// Open an existing message queue for reading and writing.
    ///
    /// In Linux, the `name` must start with a forward slash '/' and then
    /// have no other slashes in the name.
    ///
    /// Note that this will fail if the application does not have the proper
    /// permissions to access the queue.
    pub fn open(name: &str) -> Result<Self> {
        Self::open_with_flags(name, MQ_OFlag::O_RDWR)
    }

    /// Open an existing message queue with the specified flags.
    ///
    /// In Linux, the `name` must start with a forward slash '/' and then
    /// have no other slashes in the name.
    ///
    /// Note that this will fail if the application does not have the proper
    /// permissions to access the queue.
    pub fn open_with_flags(name: &str, flags: MQ_OFlag) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let mq = mqueue::mq_open(&name, flags, Mode::empty(), None)?;
        let attr = mq_getattr(mq)?;
        Ok(Self {
            mq,
            max_msg: attr.max_msg() as usize,
            msg_size: attr.msg_size() as usize,
        })
    }

    /// Create a new message queue for reading and writing with the
    /// specified sizes.
    ///
    /// Note that Linux enforces limits to these sizes that are enfroced on
    /// normal users. There a number of /proc files that show these limits,
    /// and with proper permissions, allow those limits to be changes. The
    /// files usually live under /proc/sys/fs/mqueue.
    ///
    /// Typical values might be, msg_max: 10, msgsize_max: 8192.
    pub fn create(name: &str, nmsg: usize, maxsz: usize) -> Result<Self> {
        let flags = MQ_OFlag::O_RDWR | MQ_OFlag::O_CREAT;
        let mode = Mode::from_bits_truncate(0o660);
        Self::create_with_flags(name, flags, mode, nmsg, maxsz)
    }

    /// Create a new message queue for reading and writing with the
    /// specified flags and modes.
    ///
    /// Note that this will always add the O_CREAT flag, even if not
    /// specified.
    pub fn create_with_flags(
        name: &str,
        flags: MQ_OFlag,
        mode: Mode,
        max_msg: usize,
        msg_size: usize
    ) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let flags = flags | MQ_OFlag::O_CREAT;
        let attr = mqueue::MqAttr::new(
            0,
            max_msg as mq_attr_member_t,
            msg_size as mq_attr_member_t,
            0
        );
        let mq = mqueue::mq_open(&name, flags, mode, Some(&attr))?;
        Ok(Self { mq, max_msg, msg_size })
    }

    /// Gets the maximum number of messages that can be held in the queue
    pub fn max_msg(&self) -> usize {
        self.max_msg
    }

    /// Gets the maxium size of each message in the queue
    pub const fn msg_size(&self) -> usize {
        self.msg_size
    }

    /// Sets the queue into non-blocking mode.
    ///
    /// This is a convenience function to set the O_NONBLOCK flag on the
    /// queue.
    pub fn set_nonblock(&mut self) -> Result<()> {
        mqueue::mq_set_nonblock(self.mq)?;
        Ok(())
    }

    /// Removes the queue from non-blocking mode.
    ///
    /// This is a convenience function to clear the O_NONBLOCK flag on the
    /// queue.
    pub fn remove_nonblock(&mut self) -> Result<()> {
        mqueue::mq_remove_nonblock(self.mq)?;
        Ok(())
    }

    /// Gets the attributes for the message queue
    pub fn get_attr(&self) -> Result<MqAttr> {
        mq_getattr(self.mq)
    }

    /// Sends a message to the queue with the default priority
    pub fn send(&self, msg: &[u8]) -> Result<()> {
        self.send_with_priority(msg, DEFAULT_PRIO)
    }

    /// Sends a message to the queue
    pub fn send_with_priority(&self, msg: &[u8], prio: u32) -> Result<()> {
        mqueue::mq_send(self.mq, msg, prio)
    }

    /// Receive a message
    pub fn receive(&self, msg: &mut [u8]) -> Result<usize> {
        let mut prio = 0;
        self.receive_with_priority(msg, &mut prio)
    }

    /// Receive a message
    pub fn receive_bytes(&self) -> Result<Vec<u8>> {
        let mut prio = 0;
        let mut buf = vec![0u8; self.msg_size];
        let n = self.receive_with_priority(&mut buf, &mut prio)?;
        buf.truncate(n);
        Ok(buf)
    }

    /// Receives a message from the queue with priority
    pub fn receive_with_priority(&self, msg: &mut [u8], prio: &mut u32) -> Result<usize> {
        mqueue::mq_receive(self.mq, msg, prio)
    }
}

impl Drop for MsgQueue {
    fn drop(&mut self) {
        if self.mq >= 0 {
            let _ = mqueue::mq_close(self.mq);
        }
    }
}

impl AsRawFd for MsgQueue {
    /// Gets the raw file handle for the message queue
    fn as_raw_fd(&self) -> RawFd {
        self.mq as RawFd
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    //use std::os::unix::io::AsRawFd;

    #[test]
    fn test_create_open() {
        const NAME: &str = "/rust_unit_test";
        const N: usize = 8;
        const SZ: usize = 512;

        // Create might fail if the queue exists (i.e. re-running tests)
        let mq = MsgQueue::create(NAME, N, SZ).unwrap();

        assert_eq!(N, mq.max_msg());
        assert_eq!(SZ, mq.msg_size());

        let mq = MsgQueue::open(NAME).unwrap();

        assert_eq!(N, mq.max_msg());
        assert_eq!(SZ, mq.msg_size());
    }
}

