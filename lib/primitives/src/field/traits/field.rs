use super::AdditiveGroup;
use crate::common::bits::BitIteratorBE;
use core::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::Product,
    ops::{Div, DivAssign, Neg},
};
use num_traits::{One, Zero};
use zeroize::Zeroize;

/// Defines an abstract field.
/// Types implementing [`Field`] support common field operations such as
/// addition, subtraction, multiplication, and inverses.
pub trait Field:
    'static
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Eq
    + Zero
    + One
    + Ord
    + Neg<Output = Self>
    + Zeroize
    + Sized
    + Hash
    + AdditiveGroup<Scalar = Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> DivAssign<&'a Self>
    + for<'a> Div<&'a mut Self, Output = Self>
    + for<'a> DivAssign<&'a mut Self>
    + for<'a> Product<&'a Self>
    + From<u128>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + From<i128>
    + From<i64>
    + From<i32>
    + From<i16>
    + From<i8>
    + From<bool>
    + Product<Self>
{
    /// The multiplicative identity of the field.
    const ONE: Self;

    /// Returns the extension degree of this field.
    #[must_use]
    fn extension_degree() -> usize;

    /// Returns `self * self`.
    #[must_use]
    fn square(&self) -> Self;

    /// Squares `self` in place.
    fn square_in_place(&mut self) -> &mut Self;

    /// Computes the multiplicative inverse of `self` if `self` is nonzero.
    fn inverse(&self) -> Option<Self>;

    /// If `self.inverse().is_none()`, this just returns `None`. Otherwise, it
    /// sets `self` to `self.inverse().unwrap()`.
    fn inverse_in_place(&mut self) -> Option<&mut Self>;

    /// Returns `self^exp`, where `exp` is an integer.
    ///
    /// NOTE: Consumers should pass `exp`'s type `S` with the least bit size
    /// possible.
    /// e.g. for `pow(12)` u8 type is small enough to represent `12`.
    #[must_use]
    fn pow<S: BitIteratorBE>(&self, exp: S) -> Self {
        // Variant `Option::<Self>::None` corresponds to `one`.
        // This approach removes pointless multiplications by one, that
        // are still expensive.
        let mut res: Option<Self> = None;

        for has_bit in exp.bit_be_trimmed_iter() {
            // If res is not empty, square it.
            if let Some(res) = &mut res {
                res.square_in_place();
            }

            // If bit is set,
            if has_bit {
                match res {
                    None => {
                        // and res is empty, set it to self.
                        res = Some(*self);
                    },
                    Some(ref mut res) => {
                        // and res is not empty, multiply it by self.
                        *res *= self;
                    },
                }
            }
        }

        // If res is empty, return one.
        res.unwrap_or(Self::ONE)
    }

    /// Returns `sum([a_i * b_i])`.
    #[inline]
    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }
}
