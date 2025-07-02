use super::{
    DomainCoeff, Elements, EvaluationDomain, FftField, Field, best_fft,
    bitreverse_permutation_in_place, cfg_iter_mut, k_adicity,
};
use core::{cmp::min, fmt};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct MixedRadixEvaluationDomain<F: Field> {
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

impl<F: Field> fmt::Debug for MixedRadixEvaluationDomain<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mixed-radix multiplicative subgroup of size {}", self.size)
    }
}

impl<F: FftField> EvaluationDomain<F> for MixedRadixEvaluationDomain<F> {
    type Elements = Elements<F>;

    fn new(num_coeffs: usize) -> Option<Self> {
        let size = best_mixed_domain_size::<F>(num_coeffs) as u64;
        let small_subgroup_base = F::SMALL_SUBGROUP_BASE?;

        let q = u64::from(small_subgroup_base);
        let q_adicity = k_adicity(q, size);
        let q_part = q.checked_pow(q_adicity)?;

        let two_adicity = k_adicity(2, size);
        let log_size_of_group = two_adicity;
        let two_part = 2u64.checked_pow(two_adicity)?;

        if size != q_part * two_part {
            return None;
        }

        let group_gen = F::get_root_of_unity(size)?;
        debug_assert_eq!(group_gen.pow([size].as_slice()), F::one());
        let size_as_field_element = F::from(size);
        let size_inv = size_as_field_element.inverse()?;

        Some(Self {
            size,
            log_size_of_group,
            size_as_field_element,
            size_inv,
            group_gen,
            group_gen_inv: group_gen.inverse()?,
            offset: F::one(),
            offset_inv: F::one(),
            offset_pow_size: F::one(),
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
        let small_subgroup_base = F::SMALL_SUBGROUP_BASE?;

        let num_coeffs = best_mixed_domain_size::<F>(num_coeffs) as u64;

        let q = u64::from(small_subgroup_base);
        let q_adicity = k_adicity(q, num_coeffs);
        let q_part = q.checked_pow(q_adicity)?;

        let two_adicity = k_adicity(2, num_coeffs);
        let two_part = 2u64.checked_pow(two_adicity)?;

        (num_coeffs == q_part * two_part).then_some(num_coeffs as usize)
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
        if !self.offset.is_one() {
            Self::distribute_powers(coeffs, self.offset);
        }
        coeffs.resize(self.size(), T::zero());
        best_fft(coeffs, self.group_gen, self.log_size_of_group, serial_mixed_radix_fft::<T, F>)
    }

    #[inline]
    fn ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>) {
        evals.resize(self.size(), T::zero());
        best_fft(evals, self.group_gen_inv, self.log_size_of_group, serial_mixed_radix_fft::<T, F>);
        if self.offset.is_one() {
            cfg_iter_mut!(evals).for_each(|val| *val *= self.size_inv);
        } else {
            Self::distribute_powers_and_mul_by_const(evals, self.offset_inv, self.size_inv);
        }
    }

    fn elements(&self) -> Elements<F> {
        Elements { cur_elem: self.offset, cur_pow: 0, size: self.size, group_gen: self.group_gen }
    }
}

fn mixed_radix_fft_permute(
    two_adicity: u32,
    q_adicity: u32,
    q: usize,
    n: usize,
    mut i: usize,
) -> usize {
    let mut res = 0;
    let mut shift = n;

    for _ in 0..two_adicity {
        shift /= 2;
        res += (i % 2) * shift;
        i /= 2;
    }

    for _ in 0..q_adicity {
        shift /= q;
        res += (i % q) * shift;
        i /= q;
    }

    res
}

fn best_mixed_domain_size<F: FftField>(min_size: usize) -> usize {
    let mut best = usize::MAX;
    let small_subgroup_base_adicity = F::SMALL_SUBGROUP_BASE_ADICITY.unwrap();
    let small_subgroup_base = usize::try_from(F::SMALL_SUBGROUP_BASE.unwrap()).unwrap();

    for b in 0..=small_subgroup_base_adicity {
        let mut r = small_subgroup_base.pow(b);

        let mut two_adicity = 0;
        while r < min_size {
            r *= 2;
            two_adicity += 1;
        }

        if two_adicity <= F::TWO_ADICITY {
            best = min(best, r);
        }
    }

    best
}

pub(crate) fn serial_mixed_radix_fft<T: DomainCoeff<F>, F: FftField>(
    a: &mut [T],
    omega: F,
    two_adicity: u32,
) {
    let n = a.len();
    let q = usize::try_from(F::SMALL_SUBGROUP_BASE.unwrap()).unwrap();
    let q_u64 = u64::from(F::SMALL_SUBGROUP_BASE.unwrap());
    let n_u64 = n as u64;

    let q_adicity = k_adicity(q_u64, n_u64);
    let q_part = q_u64.checked_pow(q_adicity).unwrap();
    let two_part = 2u64.checked_pow(two_adicity).unwrap();

    assert_eq!(n_u64, q_part * two_part);

    let mut m = 1;

    if q_adicity > 0 {
        let mut seen = vec![false; n];
        for k in 0..n {
            let mut i = k;
            let mut a_i = a[i];
            while !seen[i] {
                let dest = mixed_radix_fft_permute(two_adicity, q_adicity, q, n, i);

                let a_dest = a[dest];
                a[dest] = a_i;

                seen[i] = true;

                a_i = a_dest;
                i = dest;
            }
        }

        let omega_q = omega.pow([(n / q) as u64].as_slice());
        let mut qth_roots = Vec::with_capacity(q);
        qth_roots.push(F::one());
        for i in 1..q {
            qth_roots.push(qth_roots[i - 1] * omega_q);
        }

        let mut terms = vec![T::zero(); q - 1];

        for _ in 0..q_adicity {
            let w_m = omega.pow([(n / (q * m)) as u64].as_slice());
            let mut k = 0;
            while k < n {
                let mut w_j = F::one();
                for j in 0..m {
                    let base_term = a[k + j];
                    let mut w_j_i = w_j;
                    for i in 1..q {
                        terms[i - 1] = a[k + j + i * m];
                        terms[i - 1] *= w_j_i;
                        w_j_i *= w_j;
                    }

                    for i in 0..q {
                        a[k + j + i * m] = base_term;
                        for l in 1..q {
                            let mut tmp = terms[l - 1];
                            tmp *= qth_roots[(i * l) % q];
                            a[k + j + i * m] += tmp;
                        }
                    }

                    w_j *= w_m;
                }

                k += q * m;
            }
            m *= q;
        }
    } else {
        bitreverse_permutation_in_place(a, two_adicity);
    }

    for _ in 0..two_adicity {
        let w_m = omega.pow([(n / (2 * m)) as u64].as_slice());

        let mut k = 0;
        while k < n {
            let mut w = F::one();
            for j in 0..m {
                let mut t = a[(k + m) + j];
                t *= w;
                a[(k + m) + j] = a[k + j];
                a[(k + m) + j] -= t;
                a[k + j] += t;
                w *= w_m;
            }
            k += 2 * m;
        }
        m *= 2;
    }
}
