use super::{
    DenseOrSparsePolynomial, DensePolynomial, DenseUVPolynomial, EvaluationDomain, Evaluations,
    FftField, Field, Polynomial, cfg_iter_mut,
};
use alloc::{collections::BTreeMap, vec::Vec};
use core::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Deref, DerefMut, Mul, Neg, SubAssign},
};
use num_traits::Zero;

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct SparsePolynomial<F: Field> {
    coeffs: Vec<(usize, F)>,
}

impl<F: Field> fmt::Debug for SparsePolynomial<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for (i, coeff) in self.coeffs.iter().filter(|(_, c)| !c.is_zero()) {
            if *i == 0 {
                write!(f, "\n{:?}", coeff)?;
            } else if *i == 1 {
                write!(f, " + \n{:?} * x", coeff)?;
            } else {
                write!(f, " + \n{:?} * x^{}", coeff, i)?;
            }
        }
        Ok(())
    }
}

impl<F: Field> Deref for SparsePolynomial<F> {
    type Target = [(usize, F)];

    fn deref(&self) -> &[(usize, F)] {
        &self.coeffs
    }
}

impl<F: Field> DerefMut for SparsePolynomial<F> {
    fn deref_mut(&mut self) -> &mut [(usize, F)] {
        &mut self.coeffs
    }
}

impl<F: Field> Polynomial<F> for SparsePolynomial<F> {
    type Point = F;

    fn degree(&self) -> usize {
        if self.is_zero() {
            0
        } else {
            assert!(self.coeffs.last().is_some_and(|(_, c)| !c.is_zero()));
            self.coeffs.last().unwrap().0
        }
    }

    fn evaluate(&self, point: &F) -> F {
        if self.is_zero() {
            return F::zero();
        }

        let num_powers = 0usize.leading_zeros() - self.degree().leading_zeros();

        let mut powers_of_2 = Vec::with_capacity(num_powers as usize);

        let mut p = *point;

        powers_of_2.push(p);
        for _ in 1..num_powers {
            p.square_in_place();
            powers_of_2.push(p);
        }

        let total = self
            .coeffs
            .iter()
            .map(|(i, c)| {
                debug_assert_eq!(
                    F::pow_with_table(&powers_of_2[..], [*i as u64].as_slice()).unwrap(),
                    point.pow([*i as u64].as_slice()),
                    "pows not equal"
                );
                *c * F::pow_with_table(&powers_of_2[..], [*i as u64].as_slice()).unwrap()
            })
            .sum();
        total
    }
}

impl<F: Field> Add for SparsePolynomial<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        &self + &other
    }
}

impl<'a, F: Field> Add<&'a SparsePolynomial<F>> for &SparsePolynomial<F> {
    type Output = SparsePolynomial<F>;

    fn add(self, other: &'a SparsePolynomial<F>) -> SparsePolynomial<F> {
        if self.is_zero() {
            return other.clone();
        } else if other.is_zero() {
            return self.clone();
        }

        let mut result = SparsePolynomial::<F>::zero();
        let mut self_index = 0;
        let mut other_index = 0;
        loop {
            if self_index == self.coeffs.len() && other_index == other.coeffs.len() {
                return result;
            } else if self_index == self.coeffs.len() {
                result.append_coeffs(&other.coeffs[other_index..]);
                return result;
            } else if other_index == other.coeffs.len() {
                result.append_coeffs(&self.coeffs[self_index..]);
                return result;
            }
            let (self_term_degree, self_term_coeff) = self.coeffs[self_index];
            let (other_term_degree, other_term_coeff) = other.coeffs[other_index];
            match self_term_degree.cmp(&other_term_degree) {
                Ordering::Less => {
                    result.coeffs.push((self_term_degree, self_term_coeff));
                    self_index += 1;
                },
                Ordering::Equal => {
                    let term_sum = self_term_coeff + other_term_coeff;
                    if !term_sum.is_zero() {
                        result.coeffs.push((self_term_degree, term_sum));
                    }
                    self_index += 1;
                    other_index += 1;
                },
                Ordering::Greater => {
                    result.coeffs.push((other_term_degree, other_term_coeff));
                    other_index += 1;
                },
            }
        }
    }
}

impl<'a, F: Field> AddAssign<&'a Self> for SparsePolynomial<F> {
    fn add_assign(&mut self, other: &'a Self) {
        self.coeffs = (self.clone() + other.clone()).coeffs;
    }
}

impl<'a, F: Field> AddAssign<(F, &'a Self)> for SparsePolynomial<F> {
    fn add_assign(&mut self, (f, other): (F, &'a Self)) {
        self.coeffs = (self.clone() + other.clone()).coeffs;
        for i in 0..self.coeffs.len() {
            self.coeffs[i].1 *= f;
        }
    }
}

impl<F: Field> Neg for SparsePolynomial<F> {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        for (_, coeff) in &mut self.coeffs {
            *coeff = -*coeff;
        }
        self
    }
}

impl<'a, F: Field> SubAssign<&'a Self> for SparsePolynomial<F> {
    #[inline]
    fn sub_assign(&mut self, other: &'a Self) {
        let self_copy = -self.clone();
        self.coeffs = (self_copy + other.clone()).coeffs;
    }
}

impl<F: Field> Mul<F> for &SparsePolynomial<F> {
    type Output = SparsePolynomial<F>;

    #[inline]
    fn mul(self, elem: F) -> SparsePolynomial<F> {
        if self.is_zero() || elem.is_zero() {
            SparsePolynomial::zero()
        } else {
            let mut result = self.clone();
            cfg_iter_mut!(result).for_each(|e| {
                e.1 *= elem;
            });
            result
        }
    }
}

impl<F: Field> Zero for SparsePolynomial<F> {
    fn zero() -> Self {
        Self { coeffs: Vec::new() }
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty() || self.coeffs.iter().all(|(_, c)| c.is_zero())
    }
}

impl<F: Field> SparsePolynomial<F> {
    pub fn from_coefficients_slice(coeffs: &[(usize, F)]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    pub fn from_coefficients_vec(mut coeffs: Vec<(usize, F)>) -> Self {
        while coeffs.last().is_some_and(|(_, c)| c.is_zero()) {
            coeffs.pop();
        }
        coeffs.sort_by(|(c1, _), (c2, _)| c1.cmp(c2));
        assert!(coeffs.last().map_or(true, |(_, c)| !c.is_zero()));

        Self { coeffs }
    }

    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            Self::zero()
        } else {
            let mut result = BTreeMap::new();
            for (i, self_coeff) in &self.coeffs {
                for (j, other_coeff) in &other.coeffs {
                    result
                        .entry(i + j)
                        .and_modify(|cur_coeff| *cur_coeff += *self_coeff * other_coeff)
                        .or_insert_with(|| *self_coeff * other_coeff);
                }
            }
            Self::from_coefficients_vec(result.into_iter().collect())
        }
    }

    fn append_coeffs(&mut self, append_coeffs: &[(usize, F)]) {
        assert!(append_coeffs.is_empty() || self.degree() < append_coeffs[0].0);
        self.coeffs.extend_from_slice(append_coeffs);
    }
}

impl<F: FftField> SparsePolynomial<F> {
    pub fn evaluate_over_domain_by_ref<D: EvaluationDomain<F>>(
        &self,
        domain: D,
    ) -> Evaluations<F, D> {
        let poly: DenseOrSparsePolynomial<'_, F> = self.into();
        DenseOrSparsePolynomial::evaluate_over_domain(poly, domain)
    }

    pub fn evaluate_over_domain<D: EvaluationDomain<F>>(self, domain: D) -> Evaluations<F, D> {
        let poly: DenseOrSparsePolynomial<'_, F> = self.into();
        DenseOrSparsePolynomial::evaluate_over_domain(poly, domain)
    }
}

impl<F: Field> From<SparsePolynomial<F>> for DensePolynomial<F> {
    fn from(other: SparsePolynomial<F>) -> Self {
        let mut result = vec![F::zero(); other.degree() + 1];
        for (i, coeff) in other.coeffs {
            result[i] = coeff;
        }
        Self::from_coefficients_vec(result)
    }
}

impl<F: Field> From<DensePolynomial<F>> for SparsePolynomial<F> {
    fn from(dense_poly: DensePolynomial<F>) -> Self {
        Self::from_coefficients_vec(
            dense_poly
                .coeffs()
                .iter()
                .enumerate()
                .filter(|&(_, coeff)| (!coeff.is_zero()))
                .map(|(i, coeff)| (i, *coeff))
                .collect(),
        )
    }
}
