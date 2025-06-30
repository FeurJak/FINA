#![warn(unused, future_incompatible, nonstandard_style, rust_2018_idioms, rust_2021_compatibility)]
#![forbid(unsafe_code)]
mod error;
mod impls;
mod macros;
mod marshall;
pub mod serde;
mod traits;

extern crate alloc;

pub use error::*;
pub use impls::*;
pub use marshall::*;
pub use serde::*;
pub use traits::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Compress {
    Yes,
    No,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Validate {
    Yes,
    No,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct EmptyFlags;

impl Flags for EmptyFlags {
    const BIT_SIZE: usize = 0;

    #[inline]
    fn u8_bitmask(&self) -> u8 {
        0
    }

    #[inline]
    fn from_u8(_: u8) -> Option<Self> {
        Some(Self)
    }
}
