# hinix
High level *nix functionality in Rust.

This sits atop the Rust [nix](https://github.com/nix-rust/nix) crate and provides higher-order functionality for systems programming on a *nix (Linux, Posix, Unix, etc) operating system.

Initial versions of this library are mostly concerned with interprocess communications on a single host, with objects that wrap various communicaton and synchronization mechanisms such as event objects, message querues, etc.

### Unreleased Features in this Branch

- Started Posix Message Queue implementation - `MsgQueue`
- Message queue utility apps, `mqsend` & `mqrecv`

## Interprocess Communications

There are a number of objects to wrap interprocess communications mechanisms on *nix systems. These are primarily high-performance communications and synchronization subsystems in the kernel for passing data and signals between different programs.

### Posix Message Queues: `MsgQueue`

Efficient, prioritized, messaging system for multiple producers and consumers.

A process can create a named queue in the directory pathname space, regulating access through normal file permissions. Each queue is creted with a maximum number of messages and a maximum size for each message. Individual messages can be variable size, with the system properly delimiting each. Queues can be queried for size and the number of available messages, etc, and all I/O operations can be non-blocking.

See [mq_overview](https://man7.org/linux/man-pages/man7/mq_overview.7.html) man page.

This is all available via the `MsqQueue` struct. Use like:

    let mq = MsgQueue::create("/myque", 4, 512)?;
    mq.send(b"Hello, world!")?;

This creates a queue, named "/myque", with 4 slots that can contain messages up to 512 bytes each, and then writes the bytes "Hello, world!" as a message to the queue.

Note that in Linux, queue names must start with a forward slash and must not contain any other slashes.

### Event Notification: `EventFd`

A system for event notifications via the file system. This is a wait and notify system that can send events between user-space applications absed around a 64-bit integer counter. Depending on the flags used to create the object, it can act to pass values between the apps, or it can act like a semaphore where a write increments the value, a read decrements it, and the reader blocks when the value is zero.

See [eventfd](https://man7.org/linux/man-pages/man2/eventfd.2.html) man page.

A useful aspect of the event notification system is that it is based around file descriptors and thus can be inserted into I/O polling loops that use select/poll/epoll semantics to provide aditional signaling, such as when to quit or pause, or perform some other operation.

    let evt = EventFd::new(0)?;
    evt.write(42)?;
