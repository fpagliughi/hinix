# Change Log for _hinix_

## Version 0.2.1 - 2023-06-18

[Change Set](https://github.com/fpagliughi/hinix/compare/v0.2.0..v0.2.1)

Added MsgQueue and updated dependencies.

- Added support for Posix Message Queues, `MsgQueue`
- Updated to nix crate 0.26
- Re-implemented `EventFd` to use `OwnedFd` intrernally, adding support for `AsFd`.
- Added some conditional compilation to leave out features not supported on the target OS.
- Bumped MSRV to Edition 2021, v 1.63.0

## Version 0.2.0 - 2021-10-24

[Change Set](https://github.com/fpagliughi/hinix/compare/v0.1.0..v0.2.0)

This is primarily an update of the previous version to get it cleaned up and working with the latest nix crate, but as it breaks the API, it is a major version bump.

- Updated to Rust Edition 2018
- Dependent on current nix crate, v0.23
- Dropped dependency on libc
- Reformat the code with rustfmt
- [Breaking] Simplified creation of EventFd objects

## Version 0.1.0 - 2018-09-16

Initial version