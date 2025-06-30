use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use core::marker::PhantomData;
use fina_common::io::{Read, Write};

impl<T> ArkSerialize for PhantomData<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        _writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        Ok(())
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        0
    }
}

impl<T: Sync> Valid for PhantomData<T> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl<T: Send + Sync> ArkDeserialize for PhantomData<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(Self)
    }
}
