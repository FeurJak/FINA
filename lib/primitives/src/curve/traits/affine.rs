use super::{BitIteratorBE, CurveConfig, CurveGroup, Field, PrimeField};
use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, Mul, MulAssign, Neg, Sub},
};
use zeroize::Zeroize;
/// The canonical representation of an elliptic curve group element.
/// This should represent the affine coordinates of the point corresponding
/// to this group element.
///
/// The point is guaranteed to be in the correct prime order subgroup.
pub trait AffineRepr:
    Eq
    + 'static
    + Sized
    + Copy
    + Clone
    + Default
    + Send
    + Sync
    + Hash
    + Debug
    + Display
    + Zeroize
    + Neg
    + From<<Self as AffineRepr>::Group>
    + Into<<Self as AffineRepr>::Group>
    + Add<Self, Output = Self::Group>
    + for<'a> Add<&'a Self, Output = Self::Group>
    + Add<Self::Group, Output = Self::Group>
    + for<'a> Add<&'a Self::Group, Output = Self::Group>
    + Sub<Self, Output = Self::Group>
    + for<'a> Sub<&'a Self, Output = Self::Group>
    + Sub<Self::Group, Output = Self::Group>
    + for<'a> Sub<&'a Self::Group, Output = Self::Group>
    + Mul<Self::ScalarField, Output = Self::Group>
    + for<'a> Mul<&'a Self::ScalarField, Output = Self::Group>
{
    /// Associated configuration for this curve.
    type Config: CurveConfig<ScalarField = Self::ScalarField, BaseField = Self::BaseField>;

    /// Finite prime field corresponding to an appropriate prime-order subgroup
    /// of the curve group.
    type ScalarField: PrimeField;

    /// Base field that the curve is defined over.
    type BaseField: Field;

    /// The projective representation of points on this curve.
    type Group: CurveGroup<
            Config = Self::Config,
            Affine = Self,
            ScalarField = Self::ScalarField,
            BaseField = Self::BaseField,
        > + From<Self>
        + Into<Self>
        + MulAssign<Self::ScalarField>; // needed due to https://github.com/rust-lang/rust/issues/69640

    /// Returns the x and y coordinates of this affine point.
    fn xy(&self) -> Option<(Self::BaseField, Self::BaseField)>;

    /// Returns the x coordinate of this affine point.
    fn x(&self) -> Option<Self::BaseField> {
        self.xy().map(|(x, _)| x)
    }

    /// Returns the y coordinate of this affine point.
    fn y(&self) -> Option<Self::BaseField> {
        self.xy().map(|(_, y)| y)
    }

    /// Returns the point at infinity.
    fn zero() -> Self;

    /// Is `self` the point at infinity?
    fn is_zero(&self) -> bool {
        self.xy().is_none()
    }

    /// Returns a fixed generator of unknown exponent.
    #[must_use]
    fn generator() -> Self;

    /// Converts self into the projective representation.
    fn into_group(self) -> Self::Group {
        self.into()
    }

    /// Performs scalar multiplication of this element with mixed addition.
    #[must_use]
    fn mul_bigint(&self, by: impl BitIteratorBE) -> Self::Group;

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// For some curve families more efficient methods exist.
    #[must_use]
    fn clear_cofactor(&self) -> Self;

    /// Multiplies this element by the cofactor and output the
    /// resulting projective element.
    #[must_use]
    fn mul_by_cofactor_to_group(&self) -> Self::Group;

    /// Multiplies this element by the cofactor.
    #[must_use]
    fn mul_by_cofactor(&self) -> Self {
        self.mul_by_cofactor_to_group().into()
    }

    /// Multiplies this element by the inverse of the cofactor in
    /// `Self::ScalarField`.
    #[must_use]
    fn mul_by_cofactor_inv(&self) -> Self {
        self.mul_bigint(Self::Config::COFACTOR_INV.into_bigint()).into()
    }
}
