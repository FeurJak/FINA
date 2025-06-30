use ::serde::{Deserializer, Serializer};
use alloc::string::ToString;
use core::ops::{Deref, DerefMut};
use fina_common::io::{Read, Write};

use crate::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CompressedUnchecked<T>(pub T);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct UncompressedUnchecked<T>(pub T);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CompressedChecked<T>(pub T);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct UncompressedChecked<T>(pub T);

pub fn serialize<S: Serializer>(
    value: &impl ArkSerialize,
    compress: Compress,
    s: S,
) -> Result<S::Ok, S::Error> {
    use ::serde::Serialize;
    match compress {
        Compress::Yes => CompressedUnchecked(value).serialize(s),
        Compress::No => UncompressedUnchecked(value).serialize(s),
    }
}

pub fn deserialize<'de, T: ArkDeserialize, D: Deserializer<'de>>(
    d: D,
    compress: Compress,
    checked: Validate,
) -> Result<T, D::Error> {
    use ::serde::Deserialize;
    match (checked, compress) {
        (Validate::Yes, Compress::Yes) => CompressedChecked::<T>::deserialize(d).map(|val| val.0),
        (Validate::No, Compress::Yes) => CompressedUnchecked::<T>::deserialize(d).map(|val| val.0),
        (Validate::Yes, Compress::No) => UncompressedChecked::<T>::deserialize(d).map(|val| val.0),
        (Validate::No, Compress::No) => UncompressedUnchecked::<T>::deserialize(d).map(|val| val.0),
    }
}

pub fn serialize_compressed<S: Serializer>(
    value: &impl ArkSerialize,
    s: S,
) -> Result<S::Ok, S::Error> {
    serialize(value, Compress::Yes, s)
}

pub fn serialize_uncompressed<S: Serializer>(
    value: &impl ArkSerialize,
    s: S,
) -> Result<S::Ok, S::Error> {
    serialize(value, Compress::No, s)
}

pub fn deserialize_compressed_checked<'de, T: ArkDeserialize, D: Deserializer<'de>>(
    d: D,
) -> Result<T, D::Error> {
    deserialize(d, Compress::Yes, Validate::Yes)
}

pub fn deserialize_uncompressed_checked<'de, T: ArkDeserialize, D: Deserializer<'de>>(
    d: D,
) -> Result<T, D::Error> {
    deserialize(d, Compress::No, Validate::Yes)
}

pub fn deserialize_compressed_unchecked<'de, T: ArkDeserialize, D: Deserializer<'de>>(
    d: D,
) -> Result<T, D::Error> {
    deserialize(d, Compress::Yes, Validate::No)
}

pub fn deserialize_uncompressed_unchecked<'de, T: ArkDeserialize, D: Deserializer<'de>>(
    d: D,
) -> Result<T, D::Error> {
    deserialize(d, Compress::No, Validate::No)
}

macro_rules! impl_ops {
    ($type:ty, $cons:expr) => {
        impl<T> Deref for $type {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<T> From<T> for $type {
            fn from(value: T) -> $type {
                $cons(value)
            }
        }
    };
}

macro_rules! impl_serde {
    ($type:ty, $constr:ident, $compress:expr, $validate:expr, $vecmod:ident) => {
        impl<T> ::serde::Serialize for $type
        where
            T: ArkSerialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                let mut buf = alloc::vec![];
                self.0
                    .serialize_with_mode(&mut buf, $compress)
                    .map_err(|e| ::serde::ser::Error::custom(e.to_string()))?;
                serde_encoded_bytes::SliceLike::<serde_encoded_bytes::Base64>::serialize(
                    &buf, serializer,
                )
            }
        }

        impl<'de, T> ::serde::Deserialize<'de> for $type
        where
            T: ArkDeserialize,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let buf: alloc::vec::Vec<u8> = <serde_encoded_bytes::SliceLike<
                    serde_encoded_bytes::Base64,
                >>::deserialize(deserializer)?;
                let t = T::deserialize_with_mode(&buf[..], $compress, $validate)
                    .map_err(|e| ::serde::de::Error::custom(e.to_string()))?;
                Ok(Self(t))
            }
        }

        impl<T> serde_with::SerializeAs<T> for $type
        where
            T: ArkSerialize,
        {
            fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::Serialize;
                $constr(value).serialize(serializer)
            }
        }

        impl<'de, T> serde_with::DeserializeAs<'de, T> for $type
        where
            T: ArkDeserialize,
        {
            fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                use ::serde::Deserialize;
                let val: Self = Self::deserialize(deserializer)?;
                Ok(val.0)
            }
        }

        pub mod $vecmod {
            use crate::{ArkDeserialize, ArkSerialize, CompressedChecked};
            use ::serde::ser::SerializeSeq;
            use ::serde::{Deserializer, Serializer};
            use alloc::vec::Vec;
            use core::fmt;

            pub fn serialize<S, T>(
                value: &impl AsRef<[T]>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
                T: ArkSerialize,
            {
                let values = value.as_ref();
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for val in values {
                    seq.serialize_element(&CompressedChecked(val))?;
                }
                seq.end()
            }

            pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
            where
                D: Deserializer<'de>,
                T: ArkDeserialize,
            {
                struct VecArkVisitor<T> {
                    accum: Vec<T>,
                }
                impl<'de, T> serde::de::Visitor<'de> for VecArkVisitor<T>
                where
                    T: ArkDeserialize,
                {
                    type Value = Vec<T>;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(
                            formatter,
                            "a sequence of CompressedChecked<{}>",
                            core::any::type_name::<T>()
                        )
                    }

                    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        while let Some(elt) = seq.next_element::<CompressedChecked<T>>()? {
                            self.accum.push(elt.0);
                        }
                        Ok(self.accum)
                    }
                }
                deserializer.deserialize_seq(VecArkVisitor { accum: Vec::new() })
            }
        }
    };
}

macro_rules! impl_canonical {
    ($type:ty, $compress:expr, $validate:expr) => {
        impl<T: ArkSerialize> ArkSerialize for $type {
            fn serialize_with_mode<W: Write>(
                &self,
                writer: W,
                _compress: Compress,
            ) -> Result<(), SerializationError> {
                match $compress {
                    Compress::Yes => self.0.serialize_compressed(writer),
                    Compress::No => self.0.serialize_uncompressed(writer),
                }
            }

            fn serialized_size(&self, _compress: Compress) -> usize {
                self.0.serialized_size($compress)
            }
        }

        impl<T: Valid> Valid for $type {
            fn check(&self) -> Result<(), SerializationError> {
                self.0.check()
            }
        }

        impl<T: ArkDeserialize> ArkDeserialize for $type {
            fn deserialize_with_mode<R: Read>(
                reader: R,
                _compress: Compress,
                _validate: Validate,
            ) -> Result<Self, SerializationError> {
                Ok(Self(T::deserialize_with_mode(reader, $compress, $validate)?))
            }
        }
    };
}

impl_ops!(CompressedUnchecked<T>, CompressedUnchecked);
impl_ops!(UncompressedUnchecked<T>, UncompressedUnchecked);
impl_ops!(CompressedChecked<T>, CompressedChecked);
impl_ops!(UncompressedChecked<T>, UncompressedChecked);

impl_serde!(
    CompressedUnchecked<T>,
    CompressedUnchecked,
    Compress::Yes,
    Validate::No,
    vec_compressed_unchecked
);
impl_serde!(
    UncompressedUnchecked<T>,
    UncompressedUnchecked,
    Compress::No,
    Validate::No,
    vec_uncompressed_unchecked
);
impl_serde!(
    CompressedChecked<T>,
    CompressedChecked,
    Compress::Yes,
    Validate::Yes,
    vec_compressed_checked
);
impl_serde!(
    UncompressedChecked<T>,
    UncompressedChecked,
    Compress::No,
    Validate::Yes,
    vec_uncompressed_checked
);

impl_canonical!(CompressedUnchecked<T>, Compress::Yes, Validate::No);
impl_canonical!(UncompressedUnchecked<T>, Compress::No, Validate::No);
impl_canonical!(CompressedChecked<T>, Compress::Yes, Validate::Yes);
impl_canonical!(UncompressedChecked<T>, Compress::No, Validate::Yes);
