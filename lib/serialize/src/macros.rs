#[macro_export]
macro_rules! serialize_to_vec {
    ($($x:expr),*) => ({
        let mut buf = ::ark_std::vec![];
        {$crate::serialize_to_vec!(@inner buf, $($x),*)}.map(|_| buf)
    });

    (@inner $buf:expr, $y:expr, $($x:expr),*) => ({
        {
            $crate::ArkSerialize::serialize_uncompressed(&$y, &mut $buf)
        }.and({$crate::serialize_to_vec!(@inner $buf, $($x),*)})
    });

    (@inner $buf:expr, $x:expr) => ({
        $crate::ArkSerialize::serialize_uncompressed(&$x, &mut $buf)
    });
}

#[macro_export]
macro_rules! impl_ark_serialize_uint {
    ($type:ty) => {
        impl ArkSerialize for $type {
            #[inline]
            fn serialize_with_mode<W: fina_common::io::Write>(
                &self,
                mut writer: W,
                _compress: $crate::Compress,
            ) -> Result<(), $crate::error::SerializationError> {
                Ok(writer.write_all(&self.to_le_bytes())?)
            }

            #[inline]
            fn serialized_size(&self, _compress: $crate::Compress) -> usize {
                core::mem::size_of::<$type>()
            }
        }

        impl Valid for $type {
            #[inline]
            fn check(&self) -> Result<(), $crate::error::SerializationError> {
                Ok(())
            }

            #[inline]
            fn batch_check<'a>(
                _batch: impl Iterator<Item = &'a Self>,
            ) -> Result<(), $crate::error::SerializationError>
            where
                Self: 'a,
            {
                Ok(())
            }
        }

        impl ArkDeserialize for $type {
            #[inline]
            fn deserialize_with_mode<R: fina_common::io::Read>(
                mut reader: R,
                _compress: $crate::Compress,
                _validate: $crate::Validate,
            ) -> Result<Self, $crate::error::SerializationError> {
                let mut bytes = [0u8; core::mem::size_of::<$type>()];
                reader.read_exact(&mut bytes)?;
                Ok(<$type>::from_le_bytes(bytes))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_ark_serialize_tuple {
    ($( $ty: ident : $no: tt, )*) => {
        impl<$($ty, )*> Valid for ($($ty,)*) where
            $($ty: Valid,)*
        {
            #[inline]
            fn check(&self) -> Result<(), $crate::error::SerializationError> {
                $(self.$no.check()?;)*
                Ok(())
            }
        }

        #[allow(unused)]
        impl<$($ty, )*> ArkSerialize for ($($ty,)*) where
            $($ty: ArkSerialize,)*
        {
            #[inline]
            fn serialize_with_mode<W: fina_common::io::Write>(&self, mut writer: W, compress: $crate::Compress) -> Result<(), $crate::error::SerializationError> {
                $(self.$no.serialize_with_mode(&mut writer, compress)?;)*
                Ok(())
            }

            #[inline]
            fn serialized_size(&self, compress: $crate::Compress) -> usize {
                [$(
                    self.$no.serialized_size(compress),
                )*].iter().sum()
            }
        }

        impl<$($ty, )*> ArkDeserialize for ($($ty,)*) where
            $($ty: ArkDeserialize,)*
        {
            #[inline]
            #[allow(unused)]
            fn deserialize_with_mode<R: fina_common::io::Read>(mut reader: R, compress: $crate::Compress, validate: $crate::Validate) -> Result<Self, $crate::error::SerializationError> {
                Ok(($(
                    $ty::deserialize_with_mode(&mut reader, compress, validate)?,
                )*))
            }
        }
    }
}
