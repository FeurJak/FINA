pub mod bigint;
pub mod limb;
pub mod macros;
pub mod uint;

use crate::{declare_num, impl_ct_from_primitive, impl_from_primitive};

declare_num!(U64, 64);
declare_num!(U128, 128);
declare_num!(U192, 192);
declare_num!(U256, 256);
declare_num!(U384, 384);
declare_num!(U448, 448);
declare_num!(U512, 512);
declare_num!(U576, 576);
declare_num!(U640, 640);
declare_num!(U704, 704);
declare_num!(U768, 768);
declare_num!(U832, 832);

impl_ct_from_primitive!(u8, from_u8);
impl_ct_from_primitive!(u16, from_u16);
impl_ct_from_primitive!(u32, from_u32);
impl_ct_from_primitive!(u64, from_u64);
impl_ct_from_primitive!(usize, from_usize);

impl_from_primitive!(u8, from_u8);
impl_from_primitive!(u16, from_u16);
impl_from_primitive!(u32, from_u32);
impl_from_primitive!(u64, from_u64);
impl_from_primitive!(usize, from_usize);
impl_from_primitive!(u128, from_u128);
