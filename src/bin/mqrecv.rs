// hinix/src/bin/mqrecv.rs
//
// This utility application is part of the Rust 'hinix' package.
//
// Copyright (c) 2021-2023, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! This CLI application can recv a message to a Posix message queue.

use hinix::Result;

// --------------------------------------------------------------------------

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
))]
fn main() -> Result<()> {
    use clap::arg;
    use hinix::msgqueue::MsgQueue;

    // App version is package version
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let opts = clap::Command::new("mqrecv")
        .version(VERSION)
        .about("Receive messages from a Posix Message Queue")
        .arg(arg!(<name> "Name of the message queue"))
        .get_matches();

    let mut name = opts.get_one::<String>("name").unwrap().to_owned();

    if cfg!(target_os = "linux") && !name.starts_with("/") {
        name = format!("/{}", name);
    }

    // Create the queue if it doesn't already exist.
    let mq = MsgQueue::open(&name)?;

    // Read the message
    let buf = mq.receive_bytes()?;

    // Print it
    match String::from_utf8(buf) {
        Ok(s) => println!("{}", s),
        Err(err) => println!("{:?}", err.as_bytes()),
    }

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
