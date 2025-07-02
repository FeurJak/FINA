mod fft;
use super::{DomainCoeff, Elements, EvaluationDomain, FftField, Field, compute_powers_serial};
use core::fmt;
const DEGREE_AWARE_FFT_THRESHOLD_FACTOR: usize = 1 << 2;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Radix2EvaluationDomain<F: Field> {
    pub size: u64,
    pub log_size_of_group: u32,
    pub size_as_field_element: F,
    pub size_inv: F,
    pub group_gen: F,
    pub group_gen_inv: F,
    pub offset: F,
    pub offset_inv: F,
    pub offset_pow_size: F,
}

impl<F: Field> fmt::Debug for Radix2EvaluationDomain<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Radix-2 multiplicative subgroup of size {}", self.size)
    }
}

impl<F: FftField> EvaluationDomain<F> for Radix2EvaluationDomain<F> {
    type Elements = Elements<F>;

    fn new(num_coeffs: usize) -> Option<Self> {
        let size = num_coeffs.next_power_of_two() as u64;

        let log_size_of_group = size.trailing_zeros();
        if log_size_of_group > F::TWO_ADICITY {
            return None;
        }

        let group_gen = F::get_root_of_unity(size)?;
        debug_assert_eq!(group_gen.pow([size].as_slice()), F::one());
        let size_as_field_element = F::from(size);

        Some(Self {
            size,
            log_size_of_group,
            size_as_field_element,
            size_inv: size_as_field_element.inverse()?,
            group_gen,
            group_gen_inv: group_gen.inverse()?,
            offset: F::ONE,
            offset_inv: F::ONE,
            offset_pow_size: F::ONE,
        })
    }

    fn get_coset(&self, offset: F) -> Option<Self> {
        Some(Self {
            offset,
            offset_inv: offset.inverse()?,
            offset_pow_size: offset.pow([self.size].as_slice()),
            ..*self
        })
    }

    fn compute_size_of_domain(num_coeffs: usize) -> Option<usize> {
        let size = num_coeffs.checked_next_power_of_two()?;
        (size.trailing_zeros() <= F::TWO_ADICITY).then_some(size)
    }

    #[inline]
    fn size(&self) -> usize {
        self.size.try_into().unwrap()
    }

    #[inline]
    fn log_size_of_group(&self) -> u64 {
        self.log_size_of_group as u64
    }

    #[inline]
    fn size_inv(&self) -> F {
        self.size_inv
    }

    #[inline]
    fn group_gen(&self) -> F {
        self.group_gen
    }

    #[inline]
    fn group_gen_inv(&self) -> F {
        self.group_gen_inv
    }

    #[inline]
    fn coset_offset(&self) -> F {
        self.offset
    }

    #[inline]
    fn coset_offset_inv(&self) -> F {
        self.offset_inv
    }

    #[inline]
    fn coset_offset_pow_size(&self) -> F {
        self.offset_pow_size
    }

    #[inline]
    fn fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>) {
        if coeffs.len() * DEGREE_AWARE_FFT_THRESHOLD_FACTOR <= self.size() {
            self.degree_aware_fft_in_place(coeffs);
        } else {
            coeffs.resize(self.size(), T::zero());
            self.in_order_fft_in_place(coeffs);
        }
    }

    #[inline]
    fn ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>) {
        evals.resize(self.size(), T::zero());
        self.in_order_ifft_in_place(&mut *evals);
    }

    fn elements(&self) -> Elements<F> {
        Elements { cur_elem: self.offset, cur_pow: 0, size: self.size, group_gen: self.group_gen }
    }
}
