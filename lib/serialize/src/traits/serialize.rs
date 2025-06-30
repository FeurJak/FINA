use super::{Compress, HashMarshaller, SerializationError, Valid, Validate, flags::Flags};
use digest::{Digest, OutputSizeUser, generic_array::GenericArray};
use fina_common::io::{Read, Write};

pub trait ArkSerialize {
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError>;

    fn serialized_size(&self, compress: Compress) -> usize;

    fn serialize_compressed<W: Write>(&self, writer: W) -> Result<(), SerializationError> {
        self.serialize_with_mode(writer, Compress::Yes)
    }

    fn compressed_size(&self) -> usize {
        self.serialized_size(Compress::Yes)
    }

    fn serialize_uncompressed<W: Write>(&self, writer: W) -> Result<(), SerializationError> {
        self.serialize_with_mode(writer, Compress::No)
    }

    fn uncompressed_size(&self) -> usize {
        self.serialized_size(Compress::No)
    }
}

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

pub trait ArkSerializeWithFlags: ArkSerialize {
    /// Serializes `self` and `flags` into `writer`.
    fn serialize_with_flags<W: Write, F: Flags>(
        &self,
        writer: W,
        flags: F,
    ) -> Result<(), SerializationError>;
    fn serialized_size_with_flags<F: Flags>(&self) -> usize;
}
