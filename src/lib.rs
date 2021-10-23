// hinix/src/lib.rs

//#[macro_use]
//extern crate log;


pub use eventfd::*;
pub mod eventfd;

// Hinix Result type
pub type Result<T> = nix::Result<T>;
pub type Error = nix::Error;


