pub mod biguint;
pub mod bool;
pub mod btree;
pub mod generic;
pub mod isize;
pub mod list;
pub mod option;
pub mod phantom;
pub mod str;
pub mod usize;
pub mod vec;

use super::{
    Compress, Valid, Validate,
    error::SerializationError,
    impl_ark_serialize_tuple, impl_ark_serialize_uint,
    traits::{ArkDeserialize, ArkSerialize},
};
use core::borrow::Borrow;
use fina_common::io::{Read, Write};

#[inline]
fn serialize_seq<T, B, W>(
    seq: impl ExactSizeIterator<Item = B>,
    mut writer: W,
    compress: Compress,
) -> Result<(), SerializationError>
where
    T: ArkSerialize,
    B: Borrow<T>,
    W: Write,
{
    let len = seq.len() as u64;
    len.serialize_with_mode(&mut writer, compress)?;
    for item in seq {
        item.borrow().serialize_with_mode(&mut writer, compress)?;
    }
    Ok(())
}

#[inline]
fn get_serialized_size_of_seq<T, B>(
    seq: impl ExactSizeIterator<Item = B>,
    compress: Compress,
) -> usize
where
    T: ArkSerialize,
    B: Borrow<T>,
{
    8 + seq.map(|item| item.borrow().serialized_size(compress)).sum::<usize>()
}

impl_ark_serialize_uint!(u8);
impl_ark_serialize_uint!(u16);
impl_ark_serialize_uint!(u32);
impl_ark_serialize_uint!(u64);
impl_ark_serialize_uint!(i8);
impl_ark_serialize_uint!(i16);
impl_ark_serialize_uint!(i32);
impl_ark_serialize_uint!(i64);

impl_ark_serialize_tuple!();
impl_ark_serialize_tuple!(A:0,);
impl_ark_serialize_tuple!(A:0, B:1,);
impl_ark_serialize_tuple!(A:0, B:1, C:2,);
impl_ark_serialize_tuple!(A:0, B:1, C:2, D:3,);
impl_ark_serialize_tuple!(A:0, B:1, C:2, D:3, E:4,);
