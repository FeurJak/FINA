use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use alloc::collections::{BTreeMap, BTreeSet};
use fina_common::io::{Read, Write};

impl<K, V> ArkSerialize for BTreeMap<K, V>
where
    K: ArkSerialize,
    V: ArkSerialize,
{
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        let len = self.len() as u64;
        len.serialize_with_mode(&mut writer, compress)?;
        for (k, v) in self {
            k.serialize_with_mode(&mut writer, compress)?;
            v.serialize_with_mode(&mut writer, compress)?;
        }
        Ok(())
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        8 + self
            .iter()
            .map(|(k, v)| k.serialized_size(compress) + v.serialized_size(compress))
            .sum::<usize>()
    }
}

impl<K: Valid, V: Valid> Valid for BTreeMap<K, V> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        K::batch_check(self.keys())?;
        V::batch_check(self.values())
    }

    #[inline]
    fn batch_check<'a>(batch: impl Iterator<Item = &'a Self>) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        let (keys, values): (Vec<_>, Vec<_>) = batch.map(|b| (b.keys(), b.values())).unzip();
        K::batch_check(keys.into_iter().flatten())?;
        V::batch_check(values.into_iter().flatten())
    }
}

impl<K, V> ArkDeserialize for BTreeMap<K, V>
where
    K: Ord + ArkDeserialize,
    V: ArkDeserialize,
{
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let len = u64::deserialize_with_mode(&mut reader, compress, validate)?;
        (0..len)
            .map(|_| {
                Ok((
                    K::deserialize_with_mode(&mut reader, compress, validate)?,
                    V::deserialize_with_mode(&mut reader, compress, validate)?,
                ))
            })
            .collect()
    }
}

impl<V: Valid> Valid for BTreeSet<V> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        V::batch_check(self.iter())
    }

    #[inline]
    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        V::batch_check(batch.flat_map(|s| s.iter()))
    }
}

impl<V: ArkSerialize> ArkSerialize for BTreeSet<V> {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        let len = self.len() as u64;
        len.serialize_with_mode(&mut writer, compress)?;
        for v in self {
            v.serialize_with_mode(&mut writer, compress)?;
        }
        Ok(())
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        8 + self.iter().map(|v| v.serialized_size(compress)).sum::<usize>()
    }
}

impl<V> ArkDeserialize for BTreeSet<V>
where
    V: Ord + ArkDeserialize,
{
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let len = u64::deserialize_with_mode(&mut reader, compress, validate)?;
        (0..len).map(|_| V::deserialize_with_mode(&mut reader, compress, validate)).collect()
    }
}
