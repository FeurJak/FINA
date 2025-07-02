use super::{
    DomainCoeff, Elements, EvaluationDomain, FftField, Field, MixedRadixEvaluationDomain,
    Radix2EvaluationDomain, SparsePolynomial,
};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum GeneralEvaluationDomain<F: Field> {
    Radix2(Radix2EvaluationDomain<F>),
    MixedRadix(MixedRadixEvaluationDomain<F>),
}

macro_rules! map {
    ($self:expr, $f1:ident $(, $x:expr)*) => {
        match $self {
            Self::Radix2(domain) => EvaluationDomain::$f1(domain, $($x)*),
            Self::MixedRadix(domain) => EvaluationDomain::$f1(domain, $($x)*),
        }
    }
}

impl<F: FftField> EvaluationDomain<F> for GeneralEvaluationDomain<F> {
    type Elements = GeneralElements<F>;

    fn new(num_coeffs: usize) -> Option<Self> {
        Radix2EvaluationDomain::new(num_coeffs).map(Self::Radix2).or_else(|| {
            F::SMALL_SUBGROUP_BASE
                .is_some()
                .then(|| MixedRadixEvaluationDomain::new(num_coeffs).map(Self::MixedRadix))
                .flatten()
        })
    }

    fn get_coset(&self, offset: F) -> Option<Self> {
        Some(match self {
            Self::Radix2(domain) => Self::Radix2(domain.get_coset(offset)?),
            Self::MixedRadix(domain) => Self::MixedRadix(domain.get_coset(offset)?),
        })
    }

    fn compute_size_of_domain(num_coeffs: usize) -> Option<usize> {
        Radix2EvaluationDomain::<F>::compute_size_of_domain(num_coeffs).or_else(|| {
            F::SMALL_SUBGROUP_BASE
                .is_some()
                .then(|| MixedRadixEvaluationDomain::<F>::compute_size_of_domain(num_coeffs))
                .flatten()
        })
    }

    #[inline]
    fn size(&self) -> usize {
        map!(self, size)
    }

    #[inline]
    fn log_size_of_group(&self) -> u64 {
        map!(self, log_size_of_group)
    }

    #[inline]
    fn size_inv(&self) -> F {
        map!(self, size_inv)
    }

    #[inline]
    fn group_gen(&self) -> F {
        map!(self, group_gen)
    }

    #[inline]
    fn group_gen_inv(&self) -> F {
        map!(self, group_gen_inv)
    }

    #[inline]
    fn coset_offset(&self) -> F {
        map!(self, coset_offset)
    }

    #[inline]
    fn coset_offset_inv(&self) -> F {
        map!(self, coset_offset_inv)
    }

    fn coset_offset_pow_size(&self) -> F {
        map!(self, coset_offset_pow_size)
    }

    #[inline]
    fn fft_in_place<T: DomainCoeff<F>>(&self, coeffs: &mut Vec<T>) {
        map!(self, fft_in_place, coeffs)
    }

    #[inline]
    fn ifft_in_place<T: DomainCoeff<F>>(&self, evals: &mut Vec<T>) {
        map!(self, ifft_in_place, evals)
    }

    #[inline]
    fn evaluate_all_lagrange_coefficients(&self, tau: F) -> Vec<F> {
        map!(self, evaluate_all_lagrange_coefficients, tau)
    }

    #[inline]
    fn vanishing_polynomial(&self) -> SparsePolynomial<F> {
        map!(self, vanishing_polynomial)
    }

    #[inline]
    fn evaluate_vanishing_polynomial(&self, tau: F) -> F {
        map!(self, evaluate_vanishing_polynomial, tau)
    }

    fn elements(&self) -> GeneralElements<F> {
        GeneralElements(map!(self, elements))
    }
}

pub struct GeneralElements<F: FftField>(Elements<F>);

impl<F: FftField> Iterator for GeneralElements<F> {
    type Item = F;

    #[inline]
    fn next(&mut self) -> Option<F> {
        self.0.next()
    }
}
