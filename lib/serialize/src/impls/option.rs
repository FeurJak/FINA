use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use fina_common::io::{Read, Write};

impl<T: ArkSerialize> ArkSerialize for Option<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.is_some().serialize_with_mode(&mut writer, compress)?;
        if let Some(item) = self {
            item.serialize_with_mode(&mut writer, compress)?;
        }

        Ok(())
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        1 + self.as_ref().map(|s| s.serialized_size(compress)).unwrap_or(0)
    }
}

impl<T: Valid> Valid for Option<T> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        match self {
            Some(v) => v.check(),
            None => Ok(()),
        }
    }

    #[inline]
    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        T::batch_check(batch.map(Self::as_ref).filter(Option::is_some).flatten())
    }
}

impl<T: ArkDeserialize> ArkDeserialize for Option<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let is_some = bool::deserialize_with_mode(&mut reader, compress, validate)?;
        let data = is_some
            .then(|| T::deserialize_with_mode(&mut reader, compress, validate))
            .transpose()?;

        Ok(data)
    }
}
