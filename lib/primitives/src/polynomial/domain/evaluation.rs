use super::{DensePolynomial, FftField, Rng, SparsePolynomial};
use core::{
    fmt, hash,
    ops::{Add, AddAssign, MulAssign, Sub, SubAssign},
};
pub use num_traits::{One, Zero};

pub trait DomainCoeff<F: FftField>:
    Copy
    + Send
    + Sync
    + Add<Output = Self>
    + Sub<Output = Self>
    + AddAssign
    + SubAssign
    + Zero
    + MulAssign<F>
    + fmt::Debug
    + PartialEq
{
}

impl<T, F> DomainCoeff<F> for T
where
    F: FftField,
    T: Copy
        + Send
        + Sync
        + Add<Output = Self>
        + Sub<Output = Self>
        + AddAssign
        + SubAssign
        + Zero
        + MulAssign<F>
        + fmt::Debug
        + PartialEq,
{
}

pub trait EvaluationDomain<F: FftField>:
    Copy + Clone + hash::Hash + Eq + PartialEq + fmt::Debug
{
    type Elements: Iterator<Item = F> + Sized;
    fn sample_element_outside_domain<R: Rng>(&self, rng: &mut R) -> F {
        let mut t = F::rand(rng);
        while self.evaluate_vanishing_polynomial(t).is_zero() {
            t = F::rand(rng);
        }
        t
    }
    fn new(num_coeffs: usize) -> Option<Self>;
    fn new_coset(num_coeffs: usize, offset: F) -> Option<Self> {
        Self::new(num_coeffs)?.get_coset(offset)
    }
    fn get_coset(&self, offset: F) -> Option<Self>;
    fn compute_size_of_domain(num_coeffs: usize) -> Option<usize>;
    fn size(&self) -> usize;
    fn size_as_field_element(&self) -> F {
        F::from(self.size() as u64)
    }
    fn log_size_of_group(&self) -> u64;
    fn size_inv(&self) -> F;
    fn group_gen(&self) -> F;
    fn group_gen_inv(&self) -> F;
    fn coset_offset(&self) -> F;
    fn coset_offset_inv(&self) -> F;
    fn coset_offset_pow_size(&self) -> F;
    #[inline]
    fn fft<T: DomainCoeff<F>>(&self, coeffs: &[T]) -> Vec<T> {
        let mut coeffs = coeffs.to_vec();
        self.fft_in_place(&mut coeffs);
        coeffs
    }
    fn fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>);
    #[inline]
    fn ifft<T: DomainCoeff<F>>(&self, evals: &[T]) -> Vec<T> {
        let mut evals = evals.to_vec();
        self.ifft_in_place(&mut evals);
        evals
    }
    fn ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>);
    fn distribute_powers<T: DomainCoeff<F>>(coeffs: &mut [T], g: F) {
        Self::distribute_powers_and_mul_by_const(coeffs, g, F::one());
    }
    fn distribute_powers_and_mul_by_const<T: DomainCoeff<F>>(coeffs: &mut [T], g: F, c: F) {
        let mut pow = c;
        coeffs.iter_mut().for_each(|coeff| {
            *coeff *= pow;
            pow *= &g
        })
    }
    fn evaluate_all_lagrange_coefficients(&self, tau: F) -> Vec<F> {
        let size = self.size();
        let z_h_at_tau = self.evaluate_vanishing_polynomial(tau);
        let offset = self.coset_offset();
        let group_gen = self.group_gen();
        if z_h_at_tau.is_zero() {
            let mut u = vec![F::zero(); size];
            let mut omega_i = offset;
            for u_i in u.iter_mut().take(size) {
                if omega_i == tau {
                    *u_i = F::one();
                    break;
                }
                omega_i *= &group_gen;
            }
            u
        } else {
            use super::batch_inversion;

            let group_gen_inv = self.group_gen_inv();
            let v_0_inv = self.size_as_field_element() * offset.pow([size as u64 - 1].as_slice());
            let mut l_i = z_h_at_tau.inverse().unwrap() * v_0_inv;
            let mut negative_cur_elem = -offset;
            let mut lagrange_coefficients_inverse = vec![F::zero(); size];
            for coeff in &mut lagrange_coefficients_inverse {
                let r_i = tau + negative_cur_elem;
                *coeff = l_i * r_i;
                l_i *= &group_gen_inv;
                negative_cur_elem *= &group_gen;
            }
            batch_inversion(lagrange_coefficients_inverse.as_mut_slice());
            lagrange_coefficients_inverse
        }
    }
    fn vanishing_polynomial(&self) -> SparsePolynomial<F> {
        let constant_coeff = self.coset_offset_pow_size();
        let coeffs = vec![(0, -constant_coeff), (self.size(), F::one())];
        SparsePolynomial::from_coefficients_vec(coeffs)
    }
    fn evaluate_vanishing_polynomial(&self, tau: F) -> F {
        tau.pow([self.size() as u64].as_slice()) - self.coset_offset_pow_size()
    }
    fn filter_polynomial(&self, subdomain: &Self) -> DensePolynomial<F> {
        use super::DenseOrSparsePolynomial;
        let self_vanishing_poly = DenseOrSparsePolynomial::from(
            &self.vanishing_polynomial()
                * (subdomain.size_as_field_element()
                    * subdomain.coset_offset().pow([subdomain.size() as u64].as_slice())),
        );
        let subdomain_vanishing_poly = DenseOrSparsePolynomial::from(
            &subdomain.vanishing_polynomial() * self.size_as_field_element(),
        );
        let (quotient, remainder) =
            self_vanishing_poly.divide_with_q_and_r(&subdomain_vanishing_poly).unwrap();
        assert!(remainder.is_zero());
        quotient
    }
    fn evaluate_filter_polynomial(&self, subdomain: &Self, tau: F) -> F {
        let v_subdomain_of_tau = subdomain.evaluate_vanishing_polynomial(tau);
        if v_subdomain_of_tau.is_zero() {
            F::one()
        } else {
            subdomain.size_as_field_element() * self.evaluate_vanishing_polynomial(tau)
                / (self.size_as_field_element() * v_subdomain_of_tau)
        }
    }
    fn element(&self, i: usize) -> F {
        let mut result = self.group_gen().pow([i as u64].as_slice());
        if !self.coset_offset().is_one() {
            result *= self.coset_offset()
        }
        result
    }
    fn elements(&self) -> Self::Elements;
    fn reindex_by_subdomain(&self, other: Self, index: usize) -> usize {
        assert!(self.size() >= other.size());
        let period = self.size() / other.size();
        if index < other.size() {
            index * period
        } else {
            let i = index - other.size();
            let x = period - 1;
            i + (i / x) + 1
        }
    }
    #[must_use]
    fn mul_polynomials_in_evaluation_domain(&self, self_evals: &[F], other_evals: &[F]) -> Vec<F> {
        assert_eq!(self_evals.len(), other_evals.len());
        let mut result = self_evals.to_vec();

        super::cfg_iter_mut!(result).zip(other_evals).for_each(|(a, b)| *a *= b);

        result
    }
}
