// hinix/src/bin/mqsend.rs
//
// This utility application is part of the Rust 'hinix' package.
//
// Copyright (c) 2021, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! This CLI application can send a message to a Posix message queue.

use clap::{Arg, App};
use hinix::{MsgQueue, Result};

/// The number of messages the queue can hold.
const N_MSG: usize = 4;

/// The maximum size of each message
const MAX_SZ: usize = 512;

// App version is package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// --------------------------------------------------------------------------

fn main() -> Result<()> {
    let opts = App::new("mqsend")
        .version(VERSION)
        .about("Send messages to a Posix Message Queue")
        .arg(Arg::with_name("mq_name")
            .help("Name of the message queue")
            .required(true)
            .index(1))
        .arg(Arg::with_name("msg")
            .help("The message to send to the queue")
            .required(true)
            .index(2))
        .get_matches();

    let mut name = opts.value_of("mq_name")
        .unwrap()
        .to_string();

    if !name.starts_with("/") {
        name = format!("/{}", name);
    }
    // TODO: Check for other '/' chars in name?

    let msg = opts.value_of("msg").unwrap();

    // Create the queue if it doesn't already exist.
    let mq = MsgQueue::create(&name, N_MSG, MAX_SZ)?;

    // Send the message
    mq.send(msg.as_bytes())?;

    Ok(())
}
