use super::{
    DenseOrSparsePolynomial, DenseUVPolynomial, EvaluationDomain, Evaluations, FftField, Field,
    GeneralEvaluationDomain, Polynomial, Rng, SparsePolynomial, cfg_iter_mut,
};
use alloc::vec::*;
use core::{
    fmt,
    ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, Neg, Sub, SubAssign},
};
use num_traits::Zero;

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct DensePolynomial<F: Field> {
    pub coeffs: Vec<F>,
}

impl<F: Field> Polynomial<F> for DensePolynomial<F> {
    type Point = F;

    fn degree(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            assert!(self.coeffs.last().is_some_and(|coeff| !coeff.is_zero()));
            self.coeffs.len() - 1
        }
    }

    fn evaluate(&self, point: &F) -> F {
        if self.is_zero() {
            F::zero()
        } else if point.is_zero() {
            self.coeffs[0]
        } else {
            self.internal_evaluate(point)
        }
    }
}

impl<F: Field> DensePolynomial<F> {
    #[inline]
    fn horner_evaluate(poly_coeffs: &[F], point: &F) -> F {
        poly_coeffs.iter().rfold(F::zero(), move |result, coeff| result * point + coeff)
    }

    fn internal_evaluate(&self, point: &F) -> F {
        Self::horner_evaluate(&self.coeffs, point)
    }
}

impl<F: Field> DenseUVPolynomial<F> for DensePolynomial<F> {
    fn from_coefficients_slice(coeffs: &[F]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    fn from_coefficients_vec(coeffs: Vec<F>) -> Self {
        let mut result = Self { coeffs };
        result.truncate_leading_zeros();
        assert!(result.coeffs.last().map_or(true, |coeff| !coeff.is_zero()));
        result
    }

    fn coeffs(&self) -> &[F] {
        &self.coeffs
    }

    fn rand<R: Rng>(d: usize, rng: &mut R) -> Self {
        let mut random_coeffs = Vec::new();

        if d > 0 {
            for _ in 0..=(d - 1) {
                random_coeffs.push(F::rand(rng));
            }
        }

        let mut leading_coefficient = F::rand(rng);

        while leading_coefficient.is_zero() {
            leading_coefficient = F::rand(rng);
        }

        random_coeffs.push(leading_coefficient);

        Self::from_coefficients_vec(random_coeffs)
    }
}

impl<F: FftField> DensePolynomial<F> {
    pub fn mul_by_vanishing_poly<D: EvaluationDomain<F>>(&self, domain: D) -> Self {
        let mut shifted = vec![F::zero(); domain.size()];
        shifted.extend_from_slice(&self.coeffs);
        cfg_iter_mut!(shifted).zip(&self.coeffs).for_each(|(s, c)| *s -= c);
        Self::from_coefficients_vec(shifted)
    }

    pub fn divide_by_vanishing_poly<D: EvaluationDomain<F>>(&self, domain: D) -> (Self, Self) {
        let domain_size = domain.size();

        if self.coeffs.len() < domain_size {
            (Self::zero(), self.clone())
        } else {
            let mut quotient_vec = self.coeffs[domain_size..].to_vec();
            for i in 1..(self.len() / domain_size) {
                cfg_iter_mut!(quotient_vec)
                    .zip(&self.coeffs[domain_size * (i + 1)..])
                    .for_each(|(s, c)| *s += c);
            }
            let mut remainder_vec = self.coeffs[0..domain_size].to_vec();
            cfg_iter_mut!(remainder_vec).zip(&quotient_vec).for_each(|(s, c)| *s += c);

            let quotient = Self::from_coefficients_vec(quotient_vec);
            let remainder = Self::from_coefficients_vec(remainder_vec);
            (quotient, remainder)
        }
    }
}

impl<F: Field> DensePolynomial<F> {
    fn truncate_leading_zeros(&mut self) {
        while self.coeffs.last().is_some_and(|c| c.is_zero()) {
            self.coeffs.pop();
        }
    }

    pub fn naive_mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            Self::zero()
        } else {
            let mut result = vec![F::zero(); self.degree() + other.degree() + 1];
            for (i, self_coeff) in self.coeffs.iter().enumerate() {
                for (j, other_coeff) in other.coeffs.iter().enumerate() {
                    result[i + j] += &(*self_coeff * other_coeff);
                }
            }
            Self::from_coefficients_vec(result)
        }
    }
}

impl<F: FftField> DensePolynomial<F> {
    pub fn evaluate_over_domain_by_ref<D: EvaluationDomain<F>>(
        &self,
        domain: D,
    ) -> Evaluations<F, D> {
        let poly: DenseOrSparsePolynomial<'_, F> = self.into();
        DenseOrSparsePolynomial::<F>::evaluate_over_domain(poly, domain)
    }

    pub fn evaluate_over_domain<D: EvaluationDomain<F>>(self, domain: D) -> Evaluations<F, D> {
        let poly: DenseOrSparsePolynomial<'_, F> = self.into();
        DenseOrSparsePolynomial::<F>::evaluate_over_domain(poly, domain)
    }
}

impl<F: Field> fmt::Debug for DensePolynomial<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for (i, coeff) in self.coeffs.iter().enumerate().filter(|(_, c)| !c.is_zero()) {
            if i == 0 {
                write!(f, "\n{:?}", coeff)?;
            } else if i == 1 {
                write!(f, " + \n{:?} * x", coeff)?;
            } else {
                write!(f, " + \n{:?} * x^{}", coeff, i)?;
            }
        }
        Ok(())
    }
}

impl<F: Field> Deref for DensePolynomial<F> {
    type Target = [F];

    fn deref(&self) -> &[F] {
        &self.coeffs
    }
}

impl<F: Field> DerefMut for DensePolynomial<F> {
    fn deref_mut(&mut self) -> &mut [F] {
        &mut self.coeffs
    }
}

impl<'a, F: Field> Add<&'a DensePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    fn add(self, other: &'a DensePolynomial<F>) -> DensePolynomial<F> {
        if self.is_zero() {
            return other.clone();
        }
        if other.is_zero() {
            return self.clone();
        }
        let (longer, shorter) =
            if self.degree() >= other.degree() { (self, other) } else { (other, self) };
        let mut result = longer.clone();
        cfg_iter_mut!(result).zip(&shorter.coeffs).for_each(|(a, b)| *a += b);
        result.truncate_leading_zeros();

        result
    }
}

impl<'a, F: Field> Add<&'a SparsePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn add(self, other: &'a SparsePolynomial<F>) -> DensePolynomial<F> {
        if self.is_zero() {
            return other.clone().into();
        }

        if other.is_zero() {
            return self.clone();
        }

        let mut result = self.clone();
        let additional_len = other.degree().saturating_sub(result.degree());
        result.coeffs.reserve(additional_len);
        for (pow, coeff) in other.iter() {
            if let Some(target) = result.coeffs.get_mut(*pow) {
                *target += coeff;
            } else {
                result.coeffs.extend(core::iter::repeat(F::zero()).take(pow - result.coeffs.len()));
                result.coeffs.push(*coeff);
            }
        }
        result.truncate_leading_zeros();

        result
    }
}

impl<'a, F: Field> AddAssign<&'a Self> for DensePolynomial<F> {
    fn add_assign(&mut self, other: &'a Self) {
        if other.is_zero() {
            self.truncate_leading_zeros();
            return;
        }

        if self.is_zero() {
            self.coeffs.clear();
            self.coeffs.extend_from_slice(&other.coeffs);
        } else {
            let other_coeffs_len = other.coeffs.len();
            if other_coeffs_len > self.coeffs.len() {
                self.coeffs.resize(other_coeffs_len, F::zero());
            }

            self.coeffs.iter_mut().zip(&other.coeffs).for_each(|(a, b)| *a += b);
        }
        self.truncate_leading_zeros();
    }
}

impl<'a, F: Field> AddAssign<(F, &'a Self)> for DensePolynomial<F> {
    fn add_assign(&mut self, (f, other): (F, &'a Self)) {
        if other.is_zero() {
            return;
        }
        if self.is_zero() {
            self.coeffs.clear();
            self.coeffs.extend_from_slice(&other.coeffs);
            self.coeffs.iter_mut().for_each(|c| *c *= &f);
            return;
        }
        if self.degree() < other.degree() {
            self.coeffs.resize(other.coeffs.len(), F::zero());
        }
        self.coeffs.iter_mut().zip(&other.coeffs).for_each(|(a, b)| *a += f * b);
        self.truncate_leading_zeros();
    }
}

impl<'a, F: Field> AddAssign<&'a SparsePolynomial<F>> for DensePolynomial<F> {
    #[inline]
    fn add_assign(&mut self, other: &'a SparsePolynomial<F>) {
        if other.is_zero() {
            return;
        }
        if self.is_zero() {
            self.coeffs.clear();
            self.coeffs.resize(other.degree() + 1, F::zero());
            for (i, coeff) in other.iter() {
                self.coeffs[*i] = *coeff;
            }
        } else {
            let lhs_degree = self.degree();
            let max_degree = lhs_degree.max(other.degree());
            self.coeffs.resize(max_degree + 1, F::zero());
            for (pow, coeff) in other.iter() {
                if *pow <= lhs_degree {
                    self.coeffs[*pow] += coeff;
                } else {
                    self.coeffs[*pow] = *coeff;
                }
            }
        }
        self.truncate_leading_zeros();
    }
}

impl<F: Field> Neg for DensePolynomial<F> {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        self.coeffs.iter_mut().for_each(|coeff| {
            *coeff = -*coeff;
        });
        self
    }
}

impl<'a, F: Field> Sub<&'a DensePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn sub(self, other: &'a DensePolynomial<F>) -> DensePolynomial<F> {
        let mut result = if self.is_zero() {
            let mut result = other.clone();
            result.coeffs.iter_mut().for_each(|c| *c = -(*c));
            result
        } else if other.is_zero() {
            self.clone()
        } else if self.degree() >= other.degree() {
            let mut result = self.clone();
            result.coeffs.iter_mut().zip(&other.coeffs).for_each(|(a, b)| *a -= b);
            result
        } else {
            let mut result = self.clone();
            result.coeffs.resize(other.coeffs.len(), F::zero());
            result.coeffs.iter_mut().zip(&other.coeffs).for_each(|(a, b)| *a -= b);
            result
        };
        result.truncate_leading_zeros();
        result
    }
}

impl<'a, F: Field> Sub<&'a SparsePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn sub(self, other: &'a SparsePolynomial<F>) -> DensePolynomial<F> {
        if self.is_zero() {
            let result = other.clone();
            (-result).into()
        } else if other.is_zero() {
            self.clone()
        } else {
            let mut result = self.clone();
            let mut upper_coeffs = match other.degree() > result.degree() {
                true => vec![F::zero(); other.degree() - result.degree()],
                false => Vec::new(),
            };
            for (pow, coeff) in other.iter() {
                if *pow <= result.degree() {
                    result.coeffs[*pow] -= coeff;
                } else {
                    upper_coeffs[*pow - result.degree() - 1] = -*coeff;
                }
            }
            result.coeffs.extend(upper_coeffs);
            result
        }
    }
}

impl<'a, F: Field> SubAssign<&'a Self> for DensePolynomial<F> {
    #[inline]
    fn sub_assign(&mut self, other: &'a Self) {
        if self.is_zero() {
            self.coeffs.resize(other.coeffs.len(), F::zero());
        } else if other.is_zero() {
            return;
        } else if self.degree() >= other.degree() {
        } else {
            self.coeffs.resize(other.coeffs.len(), F::zero());
        }
        self.coeffs.iter_mut().zip(&other.coeffs).for_each(|(a, b)| {
            *a -= b;
        });
        self.truncate_leading_zeros();
    }
}

impl<'a, F: Field> SubAssign<&'a SparsePolynomial<F>> for DensePolynomial<F> {
    #[inline]
    fn sub_assign(&mut self, other: &'a SparsePolynomial<F>) {
        if self.is_zero() {
            self.coeffs.truncate(0);
            self.coeffs.resize(other.degree() + 1, F::zero());

            for (i, coeff) in other.iter() {
                self.coeffs[*i] = (*coeff).neg();
            }
        } else if other.is_zero() {
        } else {
            let mut upper_coeffs = match other.degree() > self.degree() {
                true => vec![F::zero(); other.degree() - self.degree()],
                false => Vec::new(),
            };
            for (pow, coeff) in other.iter() {
                if *pow <= self.degree() {
                    self.coeffs[*pow] -= coeff;
                } else {
                    upper_coeffs[*pow - self.degree() - 1] = -*coeff;
                }
            }
            self.coeffs.extend(upper_coeffs);
        }
    }
}

impl<'a, F: Field> Div<&'a DensePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn div(self, divisor: &'a DensePolynomial<F>) -> DensePolynomial<F> {
        let a = DenseOrSparsePolynomial::from(self);
        let b = DenseOrSparsePolynomial::from(divisor);
        a.divide_with_q_and_r(&b).expect("division failed").0
    }
}

impl<F: Field> Mul<F> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn mul(self, elem: F) -> DensePolynomial<F> {
        if self.is_zero() || elem.is_zero() {
            DensePolynomial::zero()
        } else {
            let mut result = self.clone();
            cfg_iter_mut!(result).for_each(|e| {
                *e *= elem;
            });
            result
        }
    }
}

impl<F: Field> Mul<F> for DensePolynomial<F> {
    type Output = Self;

    #[inline]
    fn mul(self, elem: F) -> Self {
        &self * elem
    }
}

impl<'a, F: FftField> Mul<&'a DensePolynomial<F>> for &DensePolynomial<F> {
    type Output = DensePolynomial<F>;

    #[inline]
    fn mul(self, other: &'a DensePolynomial<F>) -> DensePolynomial<F> {
        if self.is_zero() || other.is_zero() {
            DensePolynomial::zero()
        } else {
            let domain = GeneralEvaluationDomain::new(self.coeffs.len() + other.coeffs.len() - 1)
                .expect("field is not smooth enough to construct domain");
            let mut self_evals = self.evaluate_over_domain_by_ref(domain);
            let other_evals = other.evaluate_over_domain_by_ref(domain);
            self_evals *= &other_evals;
            self_evals.interpolate()
        }
    }
}

impl<F: Field> Zero for DensePolynomial<F> {
    fn zero() -> Self {
        Self { coeffs: Vec::new() }
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty() || self.coeffs.iter().all(|coeff| coeff.is_zero())
    }
}

#[macro_export]
macro_rules! impl_dense_univariatre_poly_op {
    ($trait:ident, $method:ident, $field_bound:ident) => {
        impl<F: $field_bound> $trait<DensePolynomial<F>> for DensePolynomial<F> {
            type Output = DensePolynomial<F>;

            #[inline]
            fn $method(self, other: DensePolynomial<F>) -> DensePolynomial<F> {
                (&self).$method(&other)
            }
        }

        impl<'a, F: $field_bound> $trait<&'a DensePolynomial<F>> for DensePolynomial<F> {
            type Output = DensePolynomial<F>;

            #[inline]
            fn $method(self, other: &'a DensePolynomial<F>) -> DensePolynomial<F> {
                (&self).$method(other)
            }
        }

        impl<'a, F: $field_bound> $trait<DensePolynomial<F>> for &'a DensePolynomial<F> {
            type Output = DensePolynomial<F>;

            #[inline]
            fn $method(self, other: DensePolynomial<F>) -> DensePolynomial<F> {
                self.$method(&other)
            }
        }
    };
}

impl_dense_univariatre_poly_op!(Add, add, Field);
impl_dense_univariatre_poly_op!(Sub, sub, Field);
impl_dense_univariatre_poly_op!(Mul, mul, FftField);
impl_dense_univariatre_poly_op!(Div, div, Field);
