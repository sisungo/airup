/// An extension of [`Iterator`].
pub trait IterExt<T> {
    /// Removes *all* duplicated elements from the iterator.
    fn dedup_all(&mut self) -> Vec<T>;
}
impl<T, I> IterExt<T> for I
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn dedup_all(&mut self) -> Vec<T> {
        let mut result = Vec::new();
        self.for_each(|x| {
            if !result.contains(&x) {
                result.push(x);
            }
        });
        result
    }
}
