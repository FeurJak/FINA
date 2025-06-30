use super::Iterable;
use core::iter::Rev;

#[derive(Clone, Copy)]
pub struct Reverse<I>(pub I)
where
    I: Iterable,
    I::Iter: DoubleEndedIterator;

impl<I> Iterable for Reverse<I>
where
    I: Iterable,
    I::Iter: DoubleEndedIterator,
{
    type Item = I::Item;
    type Iter = Rev<I::Iter>;

    #[inline]
    fn iter(&self) -> Self::Iter {
        self.0.iter().rev()
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
