mod evaluation;
mod general;
mod mixed_radix;
mod radix2;

use super::univariate::{DenseOrSparsePolynomial, DensePolynomial, SparsePolynomial};
use crate::{
    cfg_iter_mut,
    curve::batch_inversion,
    field::{
        k_adicity,
        traits::{FftField, Field},
    },
    rand::Rng,
};
pub use evaluation::{DomainCoeff, EvaluationDomain};
pub use general::GeneralEvaluationDomain;
use mixed_radix::MixedRadixEvaluationDomain;
use radix2::Radix2EvaluationDomain;

/// An iterator over the elements of a domain.
pub struct Elements<F: FftField> {
    pub(crate) cur_elem: F,
    pub(crate) cur_pow: u64,
    pub(crate) size: u64,
    pub(crate) group_gen: F,
}

impl<F: FftField> Iterator for Elements<F> {
    type Item = F;
    fn next(&mut self) -> Option<F> {
        if self.cur_pow == self.size {
            None
        } else {
            let cur_elem = self.cur_elem;
            self.cur_elem *= &self.group_gen;
            self.cur_pow += 1;
            Some(cur_elem)
        }
    }
}

#[inline]
pub(crate) fn bitreverse(mut n: u32, l: u32) -> u32 {
    let mut r = 0;
    for _ in 0..l {
        r = (r << 1) | (n & 1);
        n >>= 1;
    }
    r
}

#[inline]
pub fn bitreverse_permutation_in_place<T>(a: &mut [T], width: u32) {
    let n = a.len();
    for k in 0..n {
        let rk = bitreverse(k as u32, width) as usize;
        if k < rk {
            a.swap(k, rk);
        }
    }
}

pub(crate) fn compute_powers_serial<F: Field>(size: usize, root: F) -> Vec<F> {
    compute_powers_and_mul_by_const_serial(size, root, F::one())
}

pub(crate) fn compute_powers_and_mul_by_const_serial<F: Field>(
    size: usize,
    root: F,
    c: F,
) -> Vec<F> {
    let mut value = c;
    (0..size)
        .map(|_| {
            let old_value = value;
            value *= root;
            old_value
        })
        .collect()
}

pub(crate) fn best_fft<T: DomainCoeff<F>, F: FftField>(
    a: &mut [T],
    omega: F,
    log_n: u32,
    serial_fft: fn(&mut [T], F, u32),
) {
    serial_fft(a, omega, log_n)
}
