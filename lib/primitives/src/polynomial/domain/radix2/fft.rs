use super::{
    DomainCoeff, EvaluationDomain, FftField, Radix2EvaluationDomain, compute_powers_serial,
};
use fina_common::{cfg_chunks_mut, cfg_into_iter, cfg_iter, cfg_iter_mut, log2};

const DEGREE_AWARE_FFT_THRESHOLD_FACTOR: usize = 1 << 2;

#[derive(PartialEq, Eq, Debug)]
enum FFTOrder {
    II,
    IO,
    OI,
}

impl<F: FftField> Radix2EvaluationDomain<F> {
    pub(crate) fn degree_aware_fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>) {
        if !self.offset.is_one() {
            Self::distribute_powers(&mut *coeffs, self.offset);
        }
        let n = self.size();
        let log_n = self.log_size_of_group;
        let num_coeffs = if coeffs.len().is_power_of_two() {
            coeffs.len()
        } else {
            coeffs.len().checked_next_power_of_two().unwrap()
        };
        let log_d = log2(num_coeffs);

        let duplicity_of_initials = 1 << log_n.checked_sub(log_d).expect("domain is too small");

        coeffs.resize(n, T::zero());

        for i in 0..num_coeffs as u64 {
            let ri = bitrev(i, log_n);
            if i < ri {
                coeffs.swap(i as usize, ri as usize);
            }
        }

        if duplicity_of_initials > 1 {
            cfg_chunks_mut!(coeffs, duplicity_of_initials).for_each(|chunk| {
                let v = chunk[0];
                chunk[1..].fill(v);
            });
        }

        let start_gap = duplicity_of_initials;
        self.oi_helper(&mut *coeffs, self.group_gen, start_gap);
    }

    #[allow(unused)]
    pub(crate) fn in_order_fft_in_place<T: DomainCoeff<F>>(&self, x_s: &mut [T]) {
        if !self.offset.is_one() {
            Self::distribute_powers(x_s, self.offset);
        }
        self.fft_helper_in_place(x_s, FFTOrder::II);
    }

    pub(crate) fn in_order_ifft_in_place<T: DomainCoeff<F>>(&self, x_s: &mut [T]) {
        self.ifft_helper_in_place(x_s, FFTOrder::II);
        if self.offset.is_one() {
            cfg_iter_mut!(x_s).for_each(|val| *val *= self.size_inv);
        } else {
            Self::distribute_powers_and_mul_by_const(x_s, self.offset_inv, self.size_inv);
        }
    }

    fn fft_helper_in_place<T: DomainCoeff<F>>(&self, x_s: &mut [T], ord: FFTOrder) {
        let log_len = log2(x_s.len());

        if ord == FFTOrder::OI {
            self.oi_helper(x_s, self.group_gen, 1);
        } else {
            self.io_helper(x_s, self.group_gen);
        }

        if ord == FFTOrder::II {
            derange(x_s, log_len);
        }
    }

    fn ifft_helper_in_place<T: DomainCoeff<F>>(&self, x_s: &mut [T], ord: FFTOrder) {
        let log_len = log2(x_s.len());

        if ord == FFTOrder::II {
            derange(x_s, log_len);
        }

        if ord == FFTOrder::IO {
            self.io_helper(x_s, self.group_gen_inv);
        } else {
            self.oi_helper(x_s, self.group_gen_inv, 1);
        }
    }

    pub(super) fn roots_of_unity(&self, root: F) -> Vec<F> {
        compute_powers_serial((self.size as usize) / 2, root)
    }

    #[inline(always)]
    fn butterfly_fn_io<T: DomainCoeff<F>>(((lo, hi), root): ((&mut T, &mut T), &F)) {
        let mut neg = *lo;
        neg -= *hi;

        *lo += *hi;

        *hi = neg;
        *hi *= *root;
    }

    #[inline(always)]
    fn butterfly_fn_oi<T: DomainCoeff<F>>(((lo, hi), root): ((&mut T, &mut T), &F)) {
        *hi *= *root;

        let mut neg = *lo;
        neg -= *hi;

        *lo += *hi;

        *hi = neg;
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_butterfly<T: DomainCoeff<F>, G: Fn(((&mut T, &mut T), &F)) + Copy + Sync + Send>(
        g: G,
        xi: &mut [T],
        roots: &[F],
        step: usize,
        chunk_size: usize,
        num_chunks: usize,
        max_threads: usize,
        gap: usize,
    ) {
        if xi.len() <= MIN_INPUT_SIZE_FOR_PARALLELIZATION {
            xi.chunks_mut(chunk_size).for_each(|cxi| {
                let (lo, hi) = cxi.split_at_mut(gap);
                lo.iter_mut().zip(hi).zip(roots.iter().step_by(step)).for_each(g);
            });
        } else {
            cfg_chunks_mut!(xi, chunk_size).for_each(|cxi| {
                let (lo, hi) = cxi.split_at_mut(gap);
                if gap > MIN_GAP_SIZE_FOR_PARALLELIZATION && num_chunks < max_threads {
                    cfg_iter_mut!(lo).zip(hi).zip(cfg_iter!(roots).step_by(step)).for_each(g);
                } else {
                    lo.iter_mut().zip(hi).zip(roots.iter().step_by(step)).for_each(g);
                }
            });
        }
    }

    fn io_helper<T: DomainCoeff<F>>(&self, xi: &mut [T], root: F) {
        let mut roots = self.roots_of_unity(root);
        let mut step = 1;
        let mut first = true;

        let max_threads = 1;

        let mut gap = xi.len() / 2;
        while gap > 0 {
            let chunk_size = 2 * gap;
            let num_chunks = xi.len() / chunk_size;
            if num_chunks >= MIN_NUM_CHUNKS_FOR_COMPACTION {
                if !first {
                    roots = cfg_into_iter!(roots).step_by(step * 2).collect();
                }
                step = 1;
                roots.shrink_to_fit();
            } else {
                step = num_chunks;
            }
            first = false;

            Self::apply_butterfly(
                Self::butterfly_fn_io,
                xi,
                &roots,
                step,
                chunk_size,
                num_chunks,
                max_threads,
                gap,
            );

            gap /= 2;
        }
    }

    fn oi_helper<T: DomainCoeff<F>>(&self, xi: &mut [T], root: F, start_gap: usize) {
        let roots_cache = self.roots_of_unity(root);
        let compaction_max_size = core::cmp::min(
            roots_cache.len() / 2,
            roots_cache.len() / MIN_NUM_CHUNKS_FOR_COMPACTION,
        );
        let mut compacted_roots = vec![F::default(); compaction_max_size];

        let max_threads = 1;

        let mut gap = start_gap;
        while gap < xi.len() {
            let chunk_size = 2 * gap;
            let num_chunks = xi.len() / chunk_size;

            let (roots, step) = if num_chunks >= MIN_NUM_CHUNKS_FOR_COMPACTION && gap < xi.len() / 2
            {
                cfg_iter!(roots_cache)
                    .step_by(num_chunks)
                    .zip(&mut compacted_roots[..gap])
                    .for_each(|(b, a)| *a = *b);

                (&compacted_roots[..gap], 1)
            } else {
                (&roots_cache[..], num_chunks)
            };

            Self::apply_butterfly(
                Self::butterfly_fn_oi,
                xi,
                roots,
                step,
                chunk_size,
                num_chunks,
                max_threads,
                gap,
            );

            gap *= 2;
        }
    }
}

const MIN_NUM_CHUNKS_FOR_COMPACTION: usize = 1 << 7;
const MIN_GAP_SIZE_FOR_PARALLELIZATION: usize = 1 << 10;
const MIN_INPUT_SIZE_FOR_PARALLELIZATION: usize = 1 << 10;

#[inline]
const fn bitrev(a: u64, log_len: u32) -> u64 {
    a.reverse_bits().wrapping_shr(64 - log_len)
}

fn derange<T>(xi: &mut [T], log_len: u32) {
    for idx in 1..(xi.len() as u64 - 1) {
        let ridx = bitrev(idx, log_len);
        if idx < ridx {
            xi.swap(idx as usize, ridx as usize);
        }
    }
}
