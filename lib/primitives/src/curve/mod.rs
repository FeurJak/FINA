//! This module provides common operations to work with elliptic curves.
//!
//! Abstractions and api in this module are similar to Arkworks Algebra [ark-ec
//! library].
//!
//! [ark-ec library]: https://github.com/arkworks-rs/algebra/tree/master/ec
pub mod macros;
pub mod sw;
pub mod traits;

use crate::field::traits::Field;
use alloc::vec::Vec;

/// Efficiently computes inverses of non-zero elements in the slice.
///
/// Uses Montgomery's trick to compute multiple inverses with fewer field
/// operations. Zero elements remain unchanged.
///
/// # Arguments
///
/// * `v` - Mutable slice of field elements for in-place inversion.
pub fn batch_inversion<F: Field>(v: &mut [F]) {
    batch_inversion_and_mul(v, &F::one());
}

/// Efficiently computes `coeff * v_i^(-1)` for each non-zero element.
///
/// Optimizes batch inversion by multiplying each result by a coefficient.
/// Implements Montgomery's trick in two passes to minimize field inversions.
/// Zero elements remain unchanged.
///
/// # Arguments
///
/// * `v` - Mutable slice for in-place computation.
/// * `coeff` - Coefficient to multiply each inverse by.
fn batch_inversion_and_mul<F: Field>(v: &mut [F], coeff: &F) {
    // Montgomery's Trick and Fast Implementation of Masked AES
    // Genelle, Prouff and Quisquater
    // Section 3.2
    // but with an optimization to multiply every element in the returned vector
    // by coeff.

    // First pass: compute [a, ab, abc, ...]
    let mut tmp = F::one();
    let prod: Vec<_> = v
        .iter()
        .filter(|f| !f.is_zero())
        .map(|f| {
            tmp *= f;
            tmp
        })
        .collect();

    // Invert `tmp`.
    tmp = tmp.inverse().expect("should not be zero");

    // Multiply product by coeff, so coeff will scale all inverses.
    tmp *= coeff;

    // Second pass: iterate backwards to compute inverses
    for (f, s) in v
        .iter_mut()
        // Backwards
        .rev()
        // Ignore normalized elements
        .filter(|f| !f.is_zero())
        // Backwards, skip last element, fill in one for last term.
        .zip(prod.into_iter().rev().skip(1).chain(Some(F::one())))
    {
        // tmp := tmp * f; f := tmp * s = 1/f
        let new_tmp = tmp * *f;
        *f = tmp * s;
        tmp = new_tmp;
    }
}
