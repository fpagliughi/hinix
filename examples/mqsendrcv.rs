// hinix/examples/mqsend.rs
//
// This example is part of the Rust 'hinix' package.
//
// Copyright (c) 2021, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! This example shows how to communicate with a Posix message queue.
//! Normally a queue is used to send messages between two unrelated
//! processes ! (i.e. not parent/child), but here we use two threads in
//! the same application, just as a demonstration of the

#![allow(dead_code)]

use hinix::Result;

/// The name of the message queue.
/// In Linux it must start with a forward slash, '/', and then have no
/// other slashes in the name.
const NAME: &str = "/mqsendrcv";

/// The number of messages the queue can hold.
const N_MSG: usize = 4;

/// The maximum size of each message
const MAX_SZ: usize = 512;

// --------------------------------------------------------------------------
// A process can open a message queue that was created by another application
// by using the same name, assuming that it has the proper permissions.
// Here we assume that the main thread already created the queue.

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
))]
fn rx_thr() -> Result<()> {
    use hinix::msgqueue::MsgQueue;

    println!("Started receiver");

    let mq = MsgQueue::open(NAME)?;
    let mut buf: [u8; MAX_SZ] = [0; MAX_SZ];

    loop {
        let n = mq.receive(&mut buf)?;
        let s = String::from_utf8_lossy(&buf[0..n]);

        if s == "quit" {
            break;
        }
        println!("  {}", s);
    }
    Ok(())
}

// --------------------------------------------------------------------------

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
))]
fn main() -> Result<()> {
    use hinix::msgqueue::MsgQueue;

    // Create the queue if it doesn't already exist.
    let mq = MsgQueue::create(NAME, N_MSG, MAX_SZ)?;

    // Start a receiver thread
    let thr = std::thread::spawn(rx_thr);

    // Send a couple messages
    mq.send(b"Hello!")?;
    mq.send(b"Nice to see you!")?;
    mq.send(b"quit")?;

    // Wait for the thread to exit
    if let Err(err) = thr.join() {
        eprintln!("{:?}", err);
    }

    println!("Done");
    Ok(())
}

#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
)))]
fn main() -> Result<()> {
    println!("POSIX message queues not supported on this OS");
    Ok(())
}
