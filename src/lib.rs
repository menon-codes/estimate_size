/// An iterator wrapper that estimates the size of the underlying iterator.
pub struct SizeEstimate<I> {
    iter: I,
    lower: usize,
    upper: Option<usize>,
}

impl<I: Iterator + Sized> Iterator for SizeEstimate<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.lower = self.lower.saturating_sub(1);
        self.upper = self.upper.and_then(|u| u.checked_sub(1));

        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.lower, self.upper)
    }
}

/// An extension trait for iterators that allows suggesting custom size hints.
/// Particularly useful for efficient pre-allocation of collections into Vec.
pub trait EstimateSize: Iterator + Sized {
    fn estimate_size(self, lower: usize, upper: Option<usize>) -> SizeEstimate<Self> {
        SizeEstimate {
            iter: self,
            lower,
            upper,
        }
    }

    fn estimate_exact_size(self, size: usize) -> SizeEstimate<Self> {
        self.estimate_size(size, Some(size))
    }

    fn estimate_min_size(self, lower: usize) -> SizeEstimate<Self> {
        let (_, prev_upper) = self.size_hint();
        let upper = prev_upper.map(|u| u.max(lower));
        self.estimate_size(lower, upper)
    }

    fn estimate_max_size(self, upper: Option<usize>) -> SizeEstimate<Self> {
        let (prev_lower, _) = self.size_hint();
        let lower = if let Some(u) = upper {
            prev_lower.min(u)
        } else {
            prev_lower
        };
        self.estimate_size(lower, upper)
    }
}

// Implement the trait for all iterators
impl<I: Iterator> EstimateSize for I {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underflow() {
        let bad_hint_iter = std::iter::successors(Some(0), |n| Some(n + 1).filter(|&x| x < 5))
            .estimate_size(3, Some(3));

        let vec: Vec<i32> = bad_hint_iter.collect();

        assert!(vec.capacity() >= 5);
        assert!(vec.len() >= 5);
    }

    #[test]
    fn test_estimate_exact_size() {
        let iter = (0..10).estimate_exact_size(10);
        assert_eq!(iter.size_hint(), (10, Some(10)));

        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 10);
    }

    #[test]
    fn test_estimate_min_size() {
        let iter = (0..7).estimate_min_size(10);
        assert_eq!(iter.size_hint(), (10, Some(10)));

        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 7); // Actual length is 7
    }

    #[test]
    fn test_estimate_max_size() {
        let iter = (0..15).estimate_max_size(Some(10));
        assert_eq!(iter.size_hint(), (10, Some(10))); // Lower bound is min of actual and specified

        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 15); // Actual length is 15
    }

    #[test]
    fn test_iterator_behavior() {
        let mut iter = (0..5).estimate_size(10, Some(20));

        assert_eq!(iter.size_hint(), (10, Some(20)));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.size_hint(), (9, Some(19)));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.size_hint(), (8, Some(18)));

        // Collect the rest to consume
        let remaining: Vec<_> = iter.collect();
        assert_eq!(remaining, vec![2, 3, 4]);
        // Size hint isn't checked after collection as the iterator is consumed
    }

    #[test]
    fn test_with_empty_iterator() {
        let empty_iter: std::vec::IntoIter<i32> = Vec::new().into_iter();
        let mut sized_iter = empty_iter.estimate_size(5, Some(10));

        assert_eq!(sized_iter.size_hint(), (5, Some(10)));
        assert_eq!(sized_iter.next(), None); // Iterator is empty
        assert_eq!(sized_iter.size_hint(), (4, Some(9))); // But size hint is still decremented
    }

    #[test]
    fn test_with_filter() {
        let iter = (0..20)
            .filter(|x| x % 2 == 0) // Only even numbers
            .estimate_size(8, Some(12));

        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 10); // Actually 10 even numbers
    }

    #[test]
    fn test_saturating_behavior() {
        let mut iter = (0..3).estimate_size(2, Some(2));

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.size_hint(), (1, Some(1)));

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.size_hint(), (0, Some(0)));

        assert_eq!(iter.next(), Some(2));
        // Lower bound saturates at 0, doesn't underflow
        assert_eq!(iter.size_hint(), (0, None));
    }

    #[test]
    fn test_chained_iterators() {
        let first = (0..3).estimate_size(5, Some(5));
        let second = (3..6).estimate_size(5, Some(5));
        let chained = first.chain(second);

        assert_eq!(chained.count(), 6);
    }

    #[test]
    fn test_capacity_optimization() {
        // Test if vector pre-allocation works with our size hint
        let iter = (0..100).estimate_exact_size(200);
        let vec: Vec<_> = iter.collect();

        assert_eq!(vec.len(), 100); // Actual elements
        assert!(vec.capacity() >= 100); // Capacity should be at least 100
    }

    #[test]
    fn test_none_upper_bound() {
        let iter = (0..10).estimate_size(5, None);
        assert_eq!(iter.size_hint(), (5, None));

        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 10);
    }
}
