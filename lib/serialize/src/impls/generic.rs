use super::{ArkDeserialize, ArkSerialize, Compress, SerializationError, Valid, Validate};
use alloc::{borrow::Cow, rc::Rc};
use arrayvec::ArrayVec;
use fina_common::io::{Read, Write};

impl<T: ArkSerialize> ArkSerialize for &T {
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        (*self).serialize_with_mode(writer, compress)
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        (*self).serialized_size(compress)
    }
}

impl<T: ArkSerialize> ArkSerialize for &mut T {
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        (**self).serialize_with_mode(writer, compress)
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        (**self).serialized_size(compress)
    }
}

impl<T: ?Sized + ArkSerialize + ToOwned> ArkSerialize for Rc<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.as_ref().serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.as_ref().serialized_size(compress)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T: ?Sized + ArkSerialize + ToOwned> ArkSerialize for alloc::sync::Arc<T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.as_ref().serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.as_ref().serialized_size(compress)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T: ?Sized + Valid + Sync + Send> Valid for alloc::sync::Arc<T> {
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        self.as_ref().check()
    }

    #[inline]
    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        T::batch_check(batch.map(|v| v.as_ref()))
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T: ArkDeserialize + ToOwned + Sync + Send> ArkDeserialize for alloc::sync::Arc<T> {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(Self::new(T::deserialize_with_mode(reader, compress, validate)?))
    }
}

impl<T: ?Sized + ArkSerialize + ToOwned> ArkSerialize for Cow<'_, T> {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.as_ref().serialize_with_mode(&mut writer, compress)
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.as_ref().serialized_size(compress)
    }
}

impl<T> Valid for Cow<'_, T>
where
    T: ?Sized + ToOwned + Sync + Valid + Send,
    <T as ToOwned>::Owned: ArkDeserialize + Send,
{
    #[inline]
    fn check(&self) -> Result<(), SerializationError> {
        <<T as ToOwned>::Owned>::check(&self.as_ref().to_owned())
    }

    #[inline]
    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        let t: Vec<_> = batch.map(|v| v.as_ref().to_owned()).collect();
        <<T as ToOwned>::Owned>::batch_check(t.iter())
    }
}

impl<T> ArkDeserialize for Cow<'_, T>
where
    T: ?Sized + ToOwned + Valid + Sync + Send,
    <T as ToOwned>::Owned: ArkDeserialize + Valid + Send,
{
    #[inline]
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(Cow::Owned(<T as ToOwned>::Owned::deserialize_with_mode(reader, compress, validate)?))
    }
}

impl<T: ArkSerialize, const N: usize> ArkSerialize for [T; N] {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        for item in self {
            item.serialize_with_mode(&mut writer, compress)?;
        }
        Ok(())
    }

    #[inline]
    fn serialized_size(&self, compress: Compress) -> usize {
        self.iter().map(|item| item.serialized_size(compress)).sum::<usize>()
    }
}
impl<T: ArkDeserialize, const N: usize> Valid for [T; N] {
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

impl<T: ArkDeserialize, const N: usize> ArkDeserialize for [T; N] {
    #[inline]
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let mut array = ArrayVec::<T, N>::new();
        for _ in 0..N {
            array.push(T::deserialize_with_mode(&mut reader, compress, Validate::No)?);
        }
        if validate == Validate::Yes {
            T::batch_check(array.iter())?
        }
        Ok(array.into_inner().ok().unwrap())
    }
}
