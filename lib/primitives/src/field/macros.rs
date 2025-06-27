/// Auto implements conversion from [`Fp`] to integer of type `$int`.
///
/// Conversion is available only for a single limb field elements,
/// i.e. `N = 1`.
#[macro_export]
macro_rules! impl_int_from_fp {
    ($int:ty) => {
        impl<P: $crate::field::fp::FpParams<1>> From<$crate::field::fp::Fp<P, 1>> for $int {
            fn from(other: $crate::field::fp::Fp<P, 1>) -> Self {
                let uint = $crate::field::fp::FpParams::into_bigint(other);
                let words = uint.as_limbs();
                <$int>::try_from(words[0])
                    .unwrap_or_else(|_| panic!("should convert to {}", stringify!($int)))
            }
        }
    };
}

/// This macro converts a string base-10 number to a field element.
#[macro_export]
macro_rules! fp_from_num {
    ($num:literal) => {
        $crate::field::fp::Fp::new($crate::arithmetic::uint::from_str_radix($num, 10))
    };
}

/// This macro converts a string hex number to a field element.
#[macro_export]
macro_rules! fp_from_hex {
    ($num:literal) => {{ $crate::field::fp::Fp::new($crate::arithmetic::uint::from_str_hex($num)) }};
}

/// Declare [`Fp`] types for different bit sizes.
#[macro_export]
macro_rules! declare_fp {
    ($fp:ident, $limbs:ident, $bits:expr) => {
        #[doc = "Finite field with max"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub type $fp<P> = $crate::field::fp::Fp<
            P,
            { usize::div_ceil($bits, $crate::arithmetic::limb::Limb::BITS as usize) },
        >;

        #[doc = "Number of limbs in the field with"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub const $limbs: usize =
            usize::div_ceil($bits, $crate::arithmetic::limb::Limb::BITS as usize);
    };
}

/// Auto implements conversion from unsigned integer of type `$int` to [`Fp`].
#[macro_export]
macro_rules! impl_fp_from_unsigned_int {
    ($int:ty) => {
        impl<P: $crate::field::FpParams<N>, const N: usize> From<$int>
            for $crate::field::fp::Fp<P, N>
        {
            fn from(other: $int) -> Self {
                $crate::field::fp::FpParams::from_bigint($crate::arithmetic::uint::Uint::from(
                    other,
                ))
            }
        }
    };
}

/// Auto implements conversion from signed integer of type `$int` to [`Fp`].
#[macro_export]
macro_rules! impl_fp_from_signed_int {
    ($int:ty) => {
        impl<P: FpParams<N>, const N: usize> From<$int> for $crate::field::fp::Fp<P, N> {
            fn from(other: $int) -> Self {
                let abs = other.unsigned_abs().into();
                if other.is_positive() { abs } else { -abs }
            }
        }
    };
}
