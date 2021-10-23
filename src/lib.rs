// hinix/src/lib.rs

//#[macro_use]
//extern crate log;

extern crate libc;
extern crate nix;

pub use eventfd::*;

pub mod eventfd;

// Hinix Result type
pub type Result<T> = nix::Result<T>;

pub type Error = nix::Error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
