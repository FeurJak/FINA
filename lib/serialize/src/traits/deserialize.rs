use super::{ArkSerialize, Compress, Flags, HashMarshaller, SerializationError, Valid, Validate};
use digest::{Digest, OutputSizeUser, generic_array::GenericArray};
use fina_common::io::Read;

pub trait ArkDeserialize: Valid + Sized {
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError>;

    fn deserialize_compressed<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::Yes, Validate::Yes)
    }

    fn deserialize_compressed_unchecked<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::Yes, Validate::No)
    }

    fn deserialize_uncompressed<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::No, Validate::Yes)
    }

    fn deserialize_uncompressed_unchecked<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::No, Validate::No)
    }
}

pub trait ArkDeserializeWithFlags: Sized {
    fn deserialize_with_flags<R: Read, F: Flags>(
        reader: R,
    ) -> Result<(Self, F), SerializationError>;
}

pub trait ArkSerializeHashExt: ArkSerialize {
    fn hash<H: Digest>(&self) -> GenericArray<u8, <H as OutputSizeUser>::OutputSize> {
        let mut hasher = H::new();
        self.serialize_compressed(HashMarshaller(&mut hasher))
            .expect("HashMarshaller::flush should be infaillible!");
        hasher.finalize()
    }

    fn hash_uncompressed<H: Digest>(&self) -> GenericArray<u8, <H as OutputSizeUser>::OutputSize> {
        let mut hasher = H::new();
        self.serialize_uncompressed(HashMarshaller(&mut hasher))
            .expect("HashMarshaller::flush should be infaillible!");
        hasher.finalize()
    }
}
