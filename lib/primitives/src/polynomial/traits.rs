use crate::{field::traits::Field, rand::Rng};
use core::{
    fmt::Debug,
    hash::Hash,
    ops::{Add, AddAssign, Deref, Neg, SubAssign},
};
use num_traits::Zero;

pub trait Term:
    Clone
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + Hash
    + Default
    + Debug
    + Deref<Target = [(usize, usize)]>
    + Send
    + Sync
{
    fn new(term: Vec<(usize, usize)>) -> Self;
    fn degree(&self) -> usize;
    fn vars(&self) -> Vec<usize>;
    fn powers(&self) -> Vec<usize>;
    fn is_constant(&self) -> bool;
    fn evaluate<F: Field>(&self, p: &[F]) -> F;
}

pub trait Polynomial<F: Field>:
    Sized
    + Clone
    + Debug
    + Hash
    + PartialEq
    + Eq
    + Add
    + Neg
    + Zero
    + for<'a> AddAssign<&'a Self>
    + for<'a> AddAssign<(F, &'a Self)>
    + for<'a> SubAssign<&'a Self>
{
    type Point: Sized + Clone + Ord + Debug + Sync + Hash;

    fn degree(&self) -> usize;
    fn evaluate(&self, point: &Self::Point) -> F;
}

pub trait DenseMVPolynomial<F: Field>: Polynomial<F> {
    type Term: Term;

    fn terms(&self) -> &[(F, Self::Term)];
    fn num_vars(&self) -> usize;
    fn rand<R: Rng>(d: usize, num_vars: usize, rng: &mut R) -> Self;
    fn from_coefficients_vec(num_vars: usize, terms: Vec<(F, Self::Term)>) -> Self;
    fn from_coefficients_slice(num_vars: usize, terms: &[(F, Self::Term)]) -> Self {
        Self::from_coefficients_vec(num_vars, terms.to_vec())
    }
}

pub trait DenseUVPolynomial<F: Field>: Polynomial<F, Point = F> {
    fn from_coefficients_slice(coeffs: &[F]) -> Self;
    fn from_coefficients_vec(coeffs: Vec<F>) -> Self;
    fn coeffs(&self) -> &[F];
    fn rand<R: Rng>(d: usize, rng: &mut R) -> Self;
}
