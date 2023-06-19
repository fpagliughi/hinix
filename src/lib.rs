// hinix/src/lib.rs

// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2018-2023, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! The hinix crate.
//!
//! Higher-level support for *nix systems.
//!
//! # Crate Features
//!
//! * **utils** -
//!   Whether to build command-line utilities. This brings in additional
//!   dependencies like [anyhow](https://docs.rs/anyhow/latest/anyhow/) and
//!   [clap](https://docs.rs/clap/latest/clap/)
//!

// Note that the conditional compilation choices were lifted directly from
// the nix crate for which OS each underlying, wrapped, type supports.

/// Re-export nix for any apps that want to ensure the same version
/// of the underlying library.
pub use nix;

pub mod pipe;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod eventfd;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd"
))]
pub mod msgqueue;

/// Hinix Result type
/// This is simply a re-export of the nix Result type.
pub type Result<T> = nix::Result<T>;

/// Hinix Error type.
/// This is simply a re-export of the nix Error type.
pub type Error = nix::Error;
