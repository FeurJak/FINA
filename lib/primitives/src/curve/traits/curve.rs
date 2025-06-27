use super::{Field, PrimeField};
use num_traits::Zero;

/// Elliptic curves can be represented via different "models" with varying
/// efficiency properties.
///
/// [`CurveConfig`] bundles together the types that are common
/// to all models of the given curve, namely the [`Self::BaseField`] over which
/// the curve is defined, and the [`Self::ScalarField`] defined by the
/// appropriate prime-order subgroup of the curve.
pub trait CurveConfig: Send + Sync + Sized + 'static {
    /// Base field that the curve is defined over.
    type BaseField: Field;
    /// Finite prime field corresponding to an appropriate prime-order subgroup
    /// of the curve group.
    type ScalarField: PrimeField;

    /// The cofactor of this curve, represented as a sequence of little-endian
    /// limbs.
    const COFACTOR: &'static [u64];

    /// The inverse of the cofactor.
    const COFACTOR_INV: Self::ScalarField;

    /// Returns `true` if the cofactor is one.
    fn cofactor_is_one() -> bool {
        let mut iter = Self::COFACTOR.iter();
        matches!(iter.next(), Some(1)) && iter.all(Zero::is_zero)
    }
}
