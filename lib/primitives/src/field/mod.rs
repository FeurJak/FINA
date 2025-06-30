//! This module provides common arithmetics to work with finite fields.
//! Implementations of some used fields provided in the [`instance`]
//! module.
//!
//! Abstractions and api in this module are similar to Arkworks Algebra [ark-ff
//! library].
//!
//! Here is an example operations over a prime finite field (aka Fp) with a
//! prime modulus `17` and generator element `3`.
//!
//! # Examples
//!
//! ```rust
//! use primitives::{
//!     arithmetic::uint::U64,
//!     field::{
//!         fp::{Fp64, FpParams, LIMBS_64},
//!         group::AdditiveGroup,
//!         Field,
//!     },
//!     fp_from_num,
//!     from_num,
//! };
//!
//! pub type ExampleField = Fp64<FpParam>;
//! pub struct FpParam;
//! impl FpParams<LIMBS_64> for FpParam {
//!     const MODULUS: U64 = from_num!("17");
//!     const GENERATOR: Fp64<FpParam> = fp_from_num!("3");
//! }
//!
//! # fn main() {
//! let a = ExampleField::from(9);
//! let b = ExampleField::from(10);
//!
//! assert_eq!(a, ExampleField::from(26));          // 26 =  9 mod 17
//! assert_eq!(a - b, ExampleField::from(16));      // -1 = 16 mod 17
//! assert_eq!(a + b, ExampleField::from(2));       // 19 =  2 mod 17
//! assert_eq!(a * b, ExampleField::from(5));       // 90 =  5 mod 17
//! assert_eq!(a.square(), ExampleField::from(13)); // 81 = 13 mod 17
//! assert_eq!(b.double(), ExampleField::from(3));  // 20 =  3 mod 17
//! assert_eq!(a / b, a * b.inverse().unwrap());    // need to unwrap since `b` could be 0 which is not invertible
//! # }
//! ```
//!
//! [ark-ff library]: https://github.com/arkworks-rs/algebra/tree/master/ff
pub mod fp;
pub mod macros;
pub mod traits;

use crate::{
    arithmetic::{U64, U256},
    declare_fp, fp_from_num, from_num, impl_fp_from_signed_int, impl_fp_from_unsigned_int,
    impl_int_from_fp,
};
pub use fp::FpParams;

declare_fp!(Fp64, LIMBS_64, 64);
declare_fp!(Fp128, LIMBS_128, 128);
declare_fp!(Fp192, LIMBS_192, 192);
declare_fp!(Fp256, LIMBS_256, 256);
declare_fp!(Fp320, LIMBS_320, 320);
declare_fp!(Fp384, LIMBS_384, 384);
declare_fp!(Fp448, LIMBS_448, 448);
declare_fp!(Fp512, LIMBS_512, 512);
declare_fp!(Fp576, LIMBS_576, 576);
declare_fp!(Fp640, LIMBS_640, 640);
declare_fp!(Fp704, LIMBS_704, 704);
declare_fp!(Fp768, LIMBS_768, 768);
declare_fp!(Fp832, LIMBS_832, 832);

impl_fp_from_unsigned_int!(u128);
impl_fp_from_unsigned_int!(u64);
impl_fp_from_unsigned_int!(u32);
impl_fp_from_unsigned_int!(u16);
impl_fp_from_unsigned_int!(u8);

impl_fp_from_signed_int!(i128);
impl_fp_from_signed_int!(i64);
impl_fp_from_signed_int!(i32);
impl_fp_from_signed_int!(i16);
impl_fp_from_signed_int!(i8);

impl_int_from_fp!(u128);
impl_int_from_fp!(u64);
impl_int_from_fp!(u32);
impl_int_from_fp!(u16);
impl_int_from_fp!(u8);
impl_int_from_fp!(i128);
impl_int_from_fp!(i64);
impl_int_from_fp!(i32);
impl_int_from_fp!(i16);
impl_int_from_fp!(i8);

pub type FpVesta = Fp256<VestaParam>;
pub struct VestaParam;
impl FpParams<LIMBS_256> for VestaParam {
    const GENERATOR: Fp256<VestaParam> = fp_from_num!("5");
    const MODULUS: U256 =
        from_num!("28948022309329048855892746252171976963363056481941647379679742748393362948097");
}

pub type FpBabyBear = Fp64<BabyBearParam>;
pub struct BabyBearParam;
impl FpParams<LIMBS_64> for BabyBearParam {
    const GENERATOR: Fp64<BabyBearParam> = fp_from_num!("31");
    const MODULUS: U64 = from_num!("2013265921");
}

pub type FpBLS12 = Fp256<BLS12Param>;
pub struct BLS12Param;
impl FpParams<LIMBS_256> for BLS12Param {
    const GENERATOR: Fp256<BLS12Param> = fp_from_num!("7");
    const MODULUS: U256 =
        from_num!("52435875175126190479447740508185965837690552500527637822603658699938581184513");
}

pub type FpBN256 = Fp256<BN256Param>;
pub struct BN256Param;
impl FpParams<LIMBS_256> for BN256Param {
    const GENERATOR: Fp256<BN256Param> = fp_from_num!("7");
    const MODULUS: U256 =
        from_num!("21888242871839275222246405745257275088548364400416034343698204186575808495617");
}

pub type FpGoldiLocks = Fp64<GoldiLocksParam>;
pub struct GoldiLocksParam;
impl FpParams<LIMBS_64> for GoldiLocksParam {
    const GENERATOR: Fp64<GoldiLocksParam> = fp_from_num!("7");
    const MODULUS: U64 = from_num!("18446744069414584321");
}

pub type FpPallas = Fp256<PallasParam>;
pub struct PallasParam;
impl FpParams<LIMBS_256> for PallasParam {
    const GENERATOR: Fp256<PallasParam> = fp_from_num!("5");
    const MODULUS: U256 =
        from_num!("28948022309329048855892746252171976963363056481941560715954676764349967630337");
}

/// Calculates the k-adicity of n, i.e., the number of trailing 0s in a base-k
/// representation.
pub const fn k_adicity(k: u64, mut n: u64) -> u32 {
    let mut r = 0;
    while n > 1 {
        if n % k == 0 {
            r += 1;
            n /= k;
        } else {
            return r;
        }
    }
    r
}
