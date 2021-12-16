// hinix/src/lib.rs

// This is part of the Rust 'hinix' crate
//
// Copyright (c) 2018-2020, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//#[macro_use]
//extern crate log;

pub mod eventfd;
pub use eventfd::*;

pub mod msgqueue;
pub use msgqueue::{MsgQueue, MqAttr};

/// Hinix Result type
/// This is simply a re-export of the nix Result type.
pub type Result<T> = nix::Result<T>;

/// Hinix Error type.
/// This is simply a re-export of the nix Error type.
pub type Error = nix::Error;


