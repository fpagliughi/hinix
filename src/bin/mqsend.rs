// hinix/src/bin/mqsend.rs
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

//! This CLI application can send a message to a Posix message queue.

#![allow(dead_code)]

use hinix::Result;

/// The number of messages the queue can hold.
const N_MSG: usize = 4;

/// The maximum size of each message
const MAX_SZ: usize = 512;

// App version is package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// --------------------------------------------------------------------------

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
))]
fn main() -> Result<()> {
    use clap::{arg, value_parser, ArgAction};
    use hinix::msgqueue::MsgQueue;

    let opts = clap::Command::new("mqsend")
        .version(VERSION)
        .about("Send messages to a Posix Message Queue")
        .arg(arg!(-c --create "Whether to try to create the queue").action(ArgAction::SetTrue))
        .arg(
            arg!(-n --nmsg <n> "The number of messages the queue can hold")
                .required(false)
                .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!(-s --maxsz <sz> "The maximum size of each messages")
                .required(false)
                .value_parser(value_parser!(usize)),
        )
        .arg(arg!(<name> "Name of the message queue"))
        .arg(arg!(<msg> "The message to send to the queue"))
        .get_matches();

    let mut name = opts.get_one::<String>("name").unwrap().to_owned();

    if cfg!(target_os = "linux") && !name.starts_with("/") {
        name = format!("/{}", name);
    }

    let msg = opts.get_one::<String>("msg").unwrap();

    // Create the queue if it doesn't already exist.
    let mq = if opts.is_present("create") {
        let n = *opts.get_one::<usize>("nmsg").unwrap_or(&N_MSG);

        let sz = *opts.get_one::<usize>("maxsz").unwrap_or(&MAX_SZ);

        MsgQueue::create(&name, n, sz)
    }
    else {
        MsgQueue::open(&name)
    }?;

    // Send the message
    mq.send(msg)?;

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
