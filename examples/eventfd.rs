// hinix/examples/eventfd.rs
//

extern crate hinix;

use hinix::eventfd::{EfdFlags, EventFd};
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

type Result<T> = hinix::Result<T>;

fn main() -> Result<()> {
    let evtfd = EventFd::new(0, EfdFlags::empty())?;
    println!("Got it as: {:?}", evtfd.as_raw_fd());

    let evtfd = Arc::new(evtfd);
    let threvtfd = Arc::clone(&evtfd);

    thread::spawn(move || {
        for i in 1..11 {
            thread::sleep(Duration::from_millis(1000));
            println!("Signaling the event [{}]...", i);
            threvtfd.write(i).expect("Failed writing to the event");
        }

        thread::sleep(Duration::from_millis(100));
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
