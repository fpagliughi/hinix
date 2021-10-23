// hinix/examples/eventfd.rs
//
// This example is part of the Rust 'hinix' package.
//
// Copyright (c) 2018-2020, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use hinix::eventfd::EventFd;
use std::{thread, time::Duration};

const ONE_SEC: Duration = Duration::from_secs(1);
const TEN_MS: Duration = Duration::from_secs(1);

// --------------------------------------------------------------------------

fn main() -> hinix::Result<()> {
    let evtfd = EventFd::new(0)?;
    println!("Got it as: {:?}", evtfd);

    let threvtfd = evtfd.try_clone()?;
    println!("Clone is: {:?}\n", evtfd);

    thread::spawn(move || {
        for i in 1..11 {
            thread::sleep(ONE_SEC);
            println!("Signaling the event [{}]...", i);
            threvtfd.write(i).expect("Failed writing to the event");
        }

        thread::sleep(TEN_MS);
        threvtfd.write(42).expect("Failed writing to the event");
    });

    loop {
        println!("Waiting on the event...");
        let n = evtfd.read()?;
        if n == 42 {
            break;
        }
        println!("Got the event: {}", n);
    }

    println!("Done");
    Ok(())
}
