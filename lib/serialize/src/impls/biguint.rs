use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use fina_common::io::{Read, Write};
use num_bigint::BigUint;

impl ArkSerialize for BigUint {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.to_bytes_le().serialize_with_mode(writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.to_bytes_le().serialized_size(compress)
    }
}

impl ArkDeserialize for BigUint {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(Self::from_bytes_le(&Vec::<u8>::deserialize_with_mode(reader, compress, validate)?))
    }
}

impl Valid for BigUint {
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
