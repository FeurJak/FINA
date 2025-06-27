/// Declare [`Uint`] types for different bit sizes.
#[macro_export]
macro_rules! declare_num {
    ($num:ident, $bits:expr) => {
        #[doc = "Unsigned integer with "]
        #[doc = stringify!($bits)]
        #[doc = "bits size."]
        pub type $num = $crate::arithmetic::uint::Uint<
            { usize::div_ceil($bits, $crate::arithmetic::limb::Limb::BITS as usize) },
        >;
    };
}

/// Constant implementation from primitives.
#[macro_export]
macro_rules! impl_ct_from_primitive {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> $crate::arithmetic::uint::Uint<N> {
            #[doc = "Create a [`Uint`] from"]
            #[doc = stringify!($int)]
            #[doc = "integer (constant)."]
            #[must_use]
            #[allow(clippy::cast_lossless)]
            pub const fn $func_name(val: $int) -> Self {
                assert!(N >= 1, "number of limbs must be greater than zero");
                let mut repr = Self { limbs: [0u64; N] };
                repr.limbs[0] = val as u64;
                repr
            }
        }
    };
}

/// From traits implementation for primitives.
#[macro_export]
macro_rules! impl_from_primitive {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> From<$int> for $crate::arithmetic::uint::Uint<N> {
            #[inline]
            fn from(val: $int) -> $crate::arithmetic::uint::Uint<N> {
                $crate::arithmetic::uint::Uint::<N>::$func_name(val)
            }
        }
    };
}

/// This macro converts a string base-10 number to a big integer.
#[macro_export]
macro_rules! from_num {
    ($num:literal) => {
        $crate::arithmetic::uint::from_str_radix($num, 10)
    };
}

/// This macro converts a string hex number to a big integer.
#[macro_export]
macro_rules! from_hex {
    ($num:literal) => {
        $crate::arithmetic::uint::from_str_hex($num)
    };
}
