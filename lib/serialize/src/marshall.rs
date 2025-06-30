use digest::Digest;

pub(crate) struct HashMarshaller<'a, H: Digest>(pub(crate) &'a mut H);

impl<H: Digest> fina_common::io::Write for HashMarshaller<'_, H> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> fina_common::io::Result<usize> {
        Digest::update(self.0, buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> fina_common::io::Result<()> {
        Ok(())
    }
}

#[inline]
pub const fn buffer_bit_byte_size(modulus_bits: usize) -> (usize, usize) {
    let byte_size = buffer_byte_size(modulus_bits);
    ((byte_size * 8), byte_size)
}

#[inline]
pub const fn buffer_byte_size(modulus_bits: usize) -> usize {
    modulus_bits.div_ceil(8)
}
