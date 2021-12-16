// hinix/src/bin/mqrecv.rs
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

//! This CLI application can recv a message to a Posix message queue.

use clap::{Arg, App};
use hinix::{MsgQueue, Result};

// App version is package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// --------------------------------------------------------------------------

fn main() -> Result<()> {
    let opts = App::new("mqrecv")
        .version(VERSION)
        .about("Receive messages to a Posix Message Queue")
        .arg(Arg::with_name("mq_name")
            .help("Name of the message queue")
            .required(true)
            .index(1))
        .get_matches();

    let mut name = opts.value_of("mq_name")
        .unwrap()
        .to_string();

    if !name.starts_with("/") {
        name = format!("/{}", name);
    }
    // TODO: Check for other '/' chars in name?

    // Create the queue if it doesn't already exist.
    let mq = MsgQueue::open(&name)?;

    // Read the message
    let buf = mq.receive_bytes()?;

    // See if it's a string and print it
    if let Ok(s) = String::from_utf8(buf) {
        println!("{}", s);
    }
    // TODO: Print non-string

    Ok(())
}
