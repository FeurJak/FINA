use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use fina_common::io::{Read, Write};

impl ArkSerialize for bool {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        writer.write_all(&[*self as u8])?;
        Ok(())
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        1
    }
}

impl ArkDeserialize for bool {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        match u8::deserialize_with_mode(reader, compress, validate)? {
            0u8 => Ok(false),
            1u8 => Ok(true),
            _ => Err(SerializationError::InvalidData),
        }
    }
}

impl Valid for bool {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}
