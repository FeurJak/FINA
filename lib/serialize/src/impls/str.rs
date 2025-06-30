use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use fina_common::io::{Read, Write};

impl ArkSerialize for String {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.as_bytes().serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.as_bytes().serialized_size(compress)
    }
}

impl Valid for String {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl ArkDeserialize for String {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let bytes = <Vec<u8>>::deserialize_with_mode(reader, compress, validate)?;
        Self::from_utf8(bytes).map_err(|_| SerializationError::InvalidData)
    }
}
