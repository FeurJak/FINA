use super::traits::{Field, PrimeField};
use crate::arithmetic::bigint::BigInteger;

use alloc::{vec, vec::Vec};

pub trait ToFieldVector<F: Field> {
    fn to_field_elements(&self) -> Option<Vec<F>>;
}

impl<F: Field> ToFieldVector<F> for bool {
    fn to_field_elements(&self) -> Option<Vec<F>> {
        match self {
            true => Some(vec![F::one()]),
            false => Some(vec![F::zero()]),
        }
    }
}

impl<F: PrimeField> ToFieldVector<F> for F {
    fn to_field_elements(&self) -> Option<Vec<F>> {
        Some(vec![*self])
    }
}

impl<F: Field> ToFieldVector<F> for [F] {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<F>> {
        Some(self.to_vec())
    }
}

impl<F: Field> ToFieldVector<F> for () {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<F>> {
        Some(Vec::new())
    }
}

impl<F: PrimeField> ToFieldVector<F> for [u8] {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<F>> {
        let max_size = ((F::MODULUS_BIT_SIZE - 1) / 8) as usize;
        let bigint_size = <F as PrimeField>::BigInt::NUM_LIMBS * 8;
        self.chunks(max_size)
            .map(|chunk| {
                let mut bigint = vec![0u8; bigint_size];
                bigint.iter_mut().zip(chunk).for_each(|(a, b)| *a = *b);
                F::from_bigint(BigInteger::from_bytes_le(bigint.as_slice()));
            })
            .collect()
    }
}

impl<F: PrimeField> ToFieldVector<F> for [u8; 32] {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<F>> {
        self.as_ref().to_field_elements()
    }
}

impl<F: PrimeField> ToFieldVector<F> for Vec<u8> {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<F>> {
        self.as_slice().to_field_elements()
    }
}
