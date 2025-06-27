use super::{AdditiveGroup, BitIteratorBE, Field, PrimeField};
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Represents (elements of) a group of prime order `r`.
pub trait PrimeGroup: AdditiveGroup<Scalar = Self::ScalarField> {
    /// The scalar field `F_r`, where `r` is the order of this group.
    type ScalarField: PrimeField;

    /// Returns a fixed generator of this group.
    #[must_use]
    fn generator() -> Self;

    /// Performs scalar multiplication of this element.
    #[must_use]
    fn mul_bigint(&self, other: impl BitIteratorBE) -> Self;

    /// Computes `other * self`, where `other` is a *big-endian*
    /// bit representation of some integer.
    #[must_use]
    fn mul_bits_be(&self, other: impl Iterator<Item = bool>) -> Self {
        let mut res = Self::zero();
        for b in other.skip_while(|b| !b) {
            // skip leading zeros
            res.double_in_place();
            if b {
                res += self;
            }
        }
        res
    }
}

/// An opaque representation of an elliptic curve group element that is suitable
/// for efficient group arithmetic.
///
/// The point is guaranteed to be in the correct prime order subgroup.
pub trait CurveGroup:
    PrimeGroup
    + Add<Self::Affine, Output = Self>
    + AddAssign<Self::Affine>
    + Sub<Self::Affine, Output = Self>
    + SubAssign<Self::Affine>
    + From<Self::Affine>
    + Into<Self::Affine>
    + core::iter::Sum<Self::Affine>
    + for<'a> core::iter::Sum<&'a Self::Affine>
{
    /// Associated configuration for this curve.
    type Config: super::CurveConfig<ScalarField = Self::ScalarField, BaseField = Self::BaseField>;

    /// The field over which this curve is defined.
    type BaseField: Field;

    /// The affine representation of this element.
    type Affine: super::AffineRepr<
            Config = Self::Config,
            Group = Self,
            ScalarField = Self::ScalarField,
            BaseField = Self::BaseField,
        > + From<Self>
        + Into<Self>;

    /// Type representing an element of the full elliptic curve group, not just
    /// the prime order subgroup.
    type FullGroup;

    /// Normalizes a slice of group elements into affine.
    #[must_use]
    fn normalize_batch(v: &[Self]) -> Vec<Self::Affine>;

    /// Converts `self` into the affine representation.
    fn into_affine(self) -> Self::Affine {
        self.into()
    }
}
