use super::{AdditiveGroup, k_adicity, prime::PrimeField};
use crate::{bits::BitIteratorBE, log2};
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
    type BasePrimeField: PrimeField;

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

    /// Exponentiates a field element `f` by a number represented with `u64`
    /// limbs, using a precomputed table containing as many powers of 2 of
    /// `f` as the 1 + the floor of log2 of the exponent `exp`, starting
    /// from the 1st power. That is, `powers_of_2` should equal `&[p, p^2,
    /// p^4, ..., p^(2^n)]` when `exp` has at most `n` bits.
    ///
    /// This returns `None` when a power is missing from the table.
    #[inline]
    fn pow_with_table<S: BitIteratorBE>(powers_of_2: &[Self], exp: S) -> Option<Self> {
        let mut res = Self::ONE;
        for (pow, bit) in exp.bit_be_trimmed_iter().enumerate() {
            if bit {
                res *= powers_of_2.get(pow)?;
            }
        }
        Some(res)
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

/// The interface for fields that are able to be used in FFTs.
pub trait FftField: Field {
    /// The generator of the multiplicative group of the field
    const GENERATOR: Self;

    /// Let `N` be the size of the multiplicative group defined by the field.
    /// Then `TWO_ADICITY` is the two-adicity of `N`, i.e. the integer `s`
    /// such that `N = 2^s * t` for some odd integer `t`.
    const TWO_ADICITY: u32;

    /// 2^s root of unity computed by GENERATOR^t
    const TWO_ADIC_ROOT_OF_UNITY: Self;

    /// An integer `b` such that there exists a multiplicative subgroup
    /// of size `b^k` for some integer `k`.
    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    /// The integer `k` such that there exists a multiplicative subgroup
    /// of size `Self::SMALL_SUBGROUP_BASE^k`.
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    /// GENERATOR^((MODULUS-1) / (2^s *
    /// SMALL_SUBGROUP_BASE^SMALL_SUBGROUP_BASE_ADICITY)) Used for mixed-radix
    /// FFT.
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = None;

    /// Returns the root of unity of order n, if one exists.
    /// If no small multiplicative subgroup is defined, this is the 2-adic root
    /// of unity of order n (for n a power of 2).
    /// If a small multiplicative subgroup is defined, this is the root of unity
    /// of order n for the larger subgroup generated by
    /// `FftConfig::LARGE_SUBGROUP_ROOT_OF_UNITY`
    /// (for n = 2^i * FftConfig::SMALL_SUBGROUP_BASE^j for some i, j).
    fn get_root_of_unity(n: u64) -> Option<Self> {
        let mut omega: Self;
        if let Some(large_subgroup_root_of_unity) = Self::LARGE_SUBGROUP_ROOT_OF_UNITY {
            let q = Self::SMALL_SUBGROUP_BASE.expect(
                "LARGE_SUBGROUP_ROOT_OF_UNITY should only be set in conjunction with SMALL_SUBGROUP_BASE",
            ) as u64;
            let small_subgroup_base_adicity = Self::SMALL_SUBGROUP_BASE_ADICITY.expect(
                "LARGE_SUBGROUP_ROOT_OF_UNITY should only be set in conjunction with SMALL_SUBGROUP_BASE_ADICITY",
            );

            let q_adicity = k_adicity(q, n);
            let q_part = q.checked_pow(q_adicity)?;

            let two_adicity = k_adicity(2, n);
            let two_part = 2u64.checked_pow(two_adicity)?;

            if n != two_part * q_part
                || (two_adicity > Self::TWO_ADICITY)
                || (q_adicity > small_subgroup_base_adicity)
            {
                return None;
            }

            omega = large_subgroup_root_of_unity;
            for _ in q_adicity..small_subgroup_base_adicity {
                omega = omega.pow(q);
            }

            for _ in two_adicity..Self::TWO_ADICITY {
                omega.square_in_place();
            }
        } else {
            // Compute the next power of 2.
            let size = n.next_power_of_two();
            let log_size_of_group = log2(usize::try_from(size).expect("too large"));

            if n != size || log_size_of_group > Self::TWO_ADICITY {
                return None;
            }

            // Compute the generator for the multiplicative subgroup.
            // It should be 2^(log_size_of_group) root of unity.
            omega = Self::TWO_ADIC_ROOT_OF_UNITY;
            for _ in log_size_of_group..Self::TWO_ADICITY {
                omega.square_in_place();
            }
        }
        Some(omega)
    }
}
