use super::{
    ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate,
    get_serialized_size_of_seq, serialize_seq,
};
use alloc::collections::LinkedList;
use fina_common::io::{Read, Write};

impl<T: ArkSerialize> ArkSerialize for [T] {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        serialize_seq::<T, _, _>(self.iter(), writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        get_serialized_size_of_seq::<T, _>(self.iter(), compress)
    }
}

impl<T: ArkSerialize> ArkSerialize for &[T] {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        (*self).serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        (*self).serialized_size(compress)
    }
}

impl<T: ArkSerialize> ArkSerialize for LinkedList<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        serialize_seq::<T, _, _>(self.iter(), writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        get_serialized_size_of_seq::<T, _>(self.iter(), compress)
    }
}

// Identical to Valid for Vec<T>
impl<T: Valid> Valid for LinkedList<T> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        T::batch_check(self.iter())
    }

    #[inline]
    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        T::batch_check(batch.flat_map(|v| v.iter()))
    }
}

impl<T: ArkDeserialize> ArkDeserialize for LinkedList<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let len = u64::deserialize_with_mode(&mut reader, compress, validate)?
            .try_into()
            .map_err(|_| SerializationError::NotEnoughSpace)?;
        let mut values = Self::new();
        for _ in 0..len {
            values.push_back(T::deserialize_with_mode(&mut reader, compress, Validate::No)?);
        }

        if validate == Validate::Yes {
            T::batch_check(values.iter())?
        }
        Ok(values)
    }
}
