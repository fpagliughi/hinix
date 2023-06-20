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
    use clap::{App, Arg};
    use hinix::{msgqueue::MsgQueue, Result};

    // App version is package version
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let opts = App::new("mqrecv")
        .version(VERSION)
        .about("Receive messages from a Posix Message Queue")
        .arg(
            Arg::with_name("name")
                .help("Name of the message queue")
                .required(true)
                .index(1),
        )
        .get_matches();

    let mut name = opts.value_of("name").unwrap().to_string();

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
