use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use fina_common::io::{Read, Write};

impl ArkSerialize for isize {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        Ok(writer.write_all(&(*self as i64).to_le_bytes())?)
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        core::mem::size_of::<i64>()
    }
}

impl Valid for isize {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }

    #[inline]
    fn batch_check<'a>(_batch: impl Iterator<Item = &'a Self>) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        Ok(())
    }
}

impl ArkDeserialize for isize {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        let mut bytes = [0u8; core::mem::size_of::<i64>()];
        reader.read_exact(&mut bytes)?;
        Ok(<i64>::from_le_bytes(bytes) as Self)
    }
}
