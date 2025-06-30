mod rev;
pub use rev::Reverse;

pub trait Iterable: Send + Sync {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn iter(&self) -> Self::Iter;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<I> Iterable for I
where
    I: IntoIterator + Copy + Send + Sync,
    I::IntoIter: ExactSizeIterator,
{
    type Item = <I as IntoIterator>::Item;
    type Iter = <I as IntoIterator>::IntoIter;

    fn iter(&self) -> Self::Iter {
        self.into_iter()
    }

    fn len(&self) -> usize {
        self.into_iter().len()
    }
}
