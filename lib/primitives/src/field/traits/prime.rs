use super::field::{FftField, Field};
use crate::arithmetic::bigint::BigInteger;

/// Defines an abstract prime field.
/// I.e., the field of integers of prime module [`Self::MODULUS`].
pub trait PrimeField:
    Field<BasePrimeField = Self>
    + FftField
    + From<<Self as PrimeField>::BigInt>
    + Into<<Self as PrimeField>::BigInt>
{
    /// A `BigInteger` type that can represent elements of this field.
    type BigInt: BigInteger;

    /// The modulus `p`.
    const MODULUS: Self::BigInt;

    /// The size of the modulus in bits.
    const MODULUS_BIT_SIZE: usize;

    /// Returns the characteristic of the field,
    /// in little-endian representation.
    #[must_use]
    fn characteristic() -> Self::BigInt {
        Self::MODULUS
    }

    /// Construct a prime field element from a big integer.
    fn from_bigint(repr: Self::BigInt) -> Self;

    /// Converts an element of the prime field into an integer less than
    /// [`Self::MODULUS`].
    fn into_bigint(self) -> Self::BigInt;
}
