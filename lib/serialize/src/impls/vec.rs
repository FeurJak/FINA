use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use alloc::collections::VecDeque;
use fina_common::io::{Read, Write};

impl<T: ArkSerialize> ArkSerialize for Vec<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.as_slice().serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.as_slice().serialized_size(compress)
    }
}

impl<T: Valid> Valid for Vec<T> {
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

impl<T: ArkDeserialize> ArkDeserialize for Vec<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let len = u64::deserialize_with_mode(&mut reader, compress, validate)?
            .try_into()
            .map_err(|_| SerializationError::NotEnoughSpace)?;
        let mut values = Self::with_capacity(len);
        for _ in 0..len {
            values.push(T::deserialize_with_mode(&mut reader, compress, Validate::No)?);
        }

        if validate == Validate::Yes {
            T::batch_check(values.iter())?
        }
        Ok(values)
    }
}

// Identical to Valid for Vec<T>
impl<T: Valid> Valid for VecDeque<T> {
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

// Identical to ArkSerialize for Vec<T>, except using the push_back() method
impl<T: ArkDeserialize> ArkDeserialize for VecDeque<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let len = u64::deserialize_with_mode(&mut reader, compress, validate)?
            .try_into()
            .map_err(|_| SerializationError::NotEnoughSpace)?;
        let mut values = Self::with_capacity(len);
        for _ in 0..len {
            values.push_back(T::deserialize_with_mode(&mut reader, compress, Validate::No)?);
        }

        if validate == Validate::Yes {
            T::batch_check(values.iter())?
        }
        Ok(values)
    }
}
