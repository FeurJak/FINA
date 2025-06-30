#![allow(clippy::module_name_repetitions)]
#![allow(clippy::inline_always)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::many_single_char_names)]
pub mod ark_trace;
pub mod error;
pub mod io;
pub mod iterable;
pub mod macros;
mod rng;

extern crate alloc;
extern crate core;

pub use num_traits::{One, Zero};
pub use rng::*;

#[inline(always)]
pub const fn log2(x: usize) -> u32 {
    if x == 0 {
        0
    } else if x.is_power_of_two() {
        1usize.leading_zeros() - x.leading_zeros()
    } else {
        0usize.leading_zeros() - x.leading_zeros()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cfg_macros() {
        let mut thing = vec![1, 2, 3, 4, 5u64];
        println!("Iterating");
        cfg_iter!(&thing).for_each(|i| println!("{:?}", i));
        println!("Iterating Mut");
        cfg_iter_mut!(&mut thing).for_each(|i| *i += 1);
        println!("Iterating By Value");
        cfg_into_iter!(thing.clone()).for_each(|i| println!("{:?}", i));
        println!("Chunks");
        cfg_chunks!(&thing, 2).for_each(|chunk| println!("{:?}", chunk));
        println!("Chunks Mut");
        cfg_chunks_mut!(&mut thing, 2).for_each(|chunk| println!("{:?}", chunk));

        println!("Iterating");
        cfg_iter!(&thing, 3).for_each(|i| println!("{:?}", i));
        println!("Iterating Mut");
        cfg_iter_mut!(&mut thing, 3).for_each(|i| *i += 1);
        println!("Iterating By Value");
        cfg_into_iter!(thing, 3).for_each(|i| println!("{:?}", i));
    }
}
