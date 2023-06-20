// hinix/src/msgqueue.rs
//
// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2021-2023, Frank Pagliughi
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

use crate::{Error, Result};
use nix::{
    self,
    errno::Errno,
    mqueue::{self, mq_attr_member_t, MQ_OFlag, MqdT},
    sys::stat::Mode,
};
use std::ffi::CString;

/// Export the MqAttr struct from the nix crate.
pub use nix::mqueue::MqAttr;

/// The default priority for the Message Queue send operation.
pub const DEFAULT_PRIO: u32 = 0;

/// A Posix Message Queue
#[derive(Debug)]
pub struct MsgQueue {
    /// The OS file descriptor
    mq: Option<MqdT>,
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
        // TODO: Here for local
        let attr = mqueue::mq_getattr(&mq)?;
        Ok(Self {
            mq: Some(mq),
            max_msg: attr.maxmsg() as usize,
            msg_size: attr.msgsize() as usize,
        })
    }

    /// Create a new message queue for reading and writing with the
    /// specified sizes.
    ///
    /// Note that Linux enforces limits to these sizes that are enforced on
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
    /// specified sizes, but fail if the queue already exists.
    ///
    /// This simplyy adds O_EXCL flag to the creation flags.
    ///
    /// Note the size limits as described in create().
    pub fn create_exclusive(name: &str, nmsg: usize, maxsz: usize) -> Result<Self> {
        let flags = MQ_OFlag::O_RDWR | MQ_OFlag::O_CREAT | MQ_OFlag::O_EXCL;
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
        msg_size: usize,
    ) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let flags = flags | MQ_OFlag::O_CREAT;
        let attr = mqueue::MqAttr::new(
            0,
            max_msg as mq_attr_member_t,
            msg_size as mq_attr_member_t,
            0,
        );
        let mq = mqueue::mq_open(&name, flags, mode, Some(&attr))?;
        Ok(Self {
            mq: Some(mq),
            max_msg,
            msg_size,
        })
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
    pub fn set_nonblock(&mut self) -> Result<MqAttr> {
        match self.mq {
            Some(ref mq) => mqueue::mq_set_nonblock(mq),
            None => Err(Errno::ENOENT),
        }
    }

    /// Removes the queue from non-blocking mode.
    ///
    /// This is a convenience function to clear the O_NONBLOCK flag on the
    /// queue.
    pub fn remove_nonblock(&mut self) -> Result<MqAttr> {
        match self.mq {
            Some(ref mq) => mqueue::mq_remove_nonblock(mq),
            None => Err(Errno::ENOENT),
        }
    }

    /// Gets the attributes for the message queue
    pub fn get_attr(&self) -> Result<MqAttr> {
        // TODO: Here for local
        match &self.mq {
            Some(mq) => mqueue::mq_getattr(mq),
            None => Err(Errno::ENOENT),
        }
    }

    /// Sends a message to the queue with the default priority
    pub fn send<M>(&self, msg: M) -> Result<()>
    where
        M: AsRef<[u8]>,
    {
        self.send_with_priority(msg.as_ref(), DEFAULT_PRIO)
    }

    /// Sends a message to the queue
    pub fn send_with_priority<M>(&self, msg: M, prio: u32) -> Result<()>
    where
        M: AsRef<[u8]>,
    {
        match self.mq {
            Some(ref mq) => mqueue::mq_send(mq, msg.as_ref(), prio),
            None => Err(Errno::ENOENT),
        }
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

    /// Receives a message as a UTF-8 string
    pub fn receive_string(&self) -> Result<String> {
        let v = self.receive_bytes()?;
        let s = String::from_utf8(v).map_err(|_| Error::EINVAL)?;
        Ok(s)
    }

    /// Receives a message from the queue with priority
    pub fn receive_with_priority(&self, msg: &mut [u8], prio: &mut u32) -> Result<usize> {
        match self.mq {
            Some(ref mq) => mqueue::mq_receive(mq, msg, prio),
            None => Err(Errno::ENOENT),
        }
    }
}

impl Drop for MsgQueue {
    fn drop(&mut self) {
        if let Some(mq) = self.mq.take() {
            let _ = mqueue::mq_close(mq);
        }
    }
}

// TODO: Restore this on platforms that support it (upstream)?
/*
impl AsRawFd for MsgQueue {
    /// Gets the raw file handle for the message queue
    fn as_raw_fd(&self) -> RawFd {
        self.mq as RawFd
    }
}
*/

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    //use std::os::unix::io::AsRawFd;

    // Be careful that multiple tests are not reading/writing to the same
    // queue, since tests may be running in parallel.

    const NAME: &str = "/rust_unit_test";
    const N: usize = 8;
    const SZ: usize = 512;

    #[test]
    fn test_create_open() {
        // Create should succeed even if the queue exists, so long as the
        // sizes match (from the last run of the tests
        let mq = MsgQueue::create(NAME, N, SZ).unwrap();

        assert_eq!(N, mq.max_msg());
        assert_eq!(SZ, mq.msg_size());

        // Now that it exists, create exclusive should fail
        assert!(MsgQueue::create_exclusive(NAME, N, SZ).is_err());

        // We should be able to open it.
        let mq = MsgQueue::open(NAME).unwrap();

        assert_eq!(N, mq.max_msg());
        assert_eq!(SZ, mq.msg_size());
    }

    #[test]
    fn test_read_write() {
        let mut wr_arr = [0u8; SZ];
        let mut rd_arr = [0u8; SZ];

        for i in 0..SZ {
            wr_arr[i] = i as u8;
        }

        let mq = MsgQueue::create(NAME, N, SZ).unwrap();

        let attr = mq.get_attr().unwrap();
        let mut n = attr.curmsgs();

        while n != 0 {
            mq.receive(&mut rd_arr).unwrap();
            n -= 1;
        }

        let attr = mq.get_attr().unwrap();
        assert_eq!(attr.curmsgs(), 0);

        mq.send(&wr_arr).unwrap();

        let n = mq.receive(&mut rd_arr).unwrap();
        assert_eq!(n, SZ);
        assert_eq!(rd_arr, wr_arr);
    }

    #[test]
    fn test_read_write_string() {
        const NAME: &str = "/rust_str_unit_test";

        let mq = MsgQueue::create(NAME, N, SZ).unwrap();

        // Clear out existing messages
        // (Maybe better to unlink/rm old queue before starting
        while mq.get_attr().unwrap().curmsgs() != 0 {
            let mut rd_arr = [0u8; SZ];
            mq.receive(&mut rd_arr).unwrap();
        }

        const MSG: &str = "Hello, world!";
        mq.send(MSG).unwrap();

        let msg = mq.receive_string().unwrap();
        assert_eq!(MSG.to_string(), msg);
    }
}
