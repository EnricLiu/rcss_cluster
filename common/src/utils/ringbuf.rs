use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct OverwriteRB<T, const N: usize>(VecDeque<T>);

impl<T, const N: usize> OverwriteRB<T, N> {
    pub fn new() -> Self {
        OverwriteRB(VecDeque::with_capacity(N))
    }

    pub fn push(&mut self, item: T) -> Option<T> {
        let ret = if self.0.len() == N {
            self.0.pop_front()
        } else {
            None
        };
        self.0.push_back(item);

        ret
    }

    pub fn push_many<I>(&mut self, items: I) -> usize
    where I: DoubleEndedIterator<Item = T> + ExactSizeIterator
    {
        let latest_n = items.into_iter().rev().take(N);
        let mut ret = 0;
        for item in latest_n.rev() {
            self.push(item);
            ret += 1;
        }

        ret
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn to_vec(&self) -> Vec<T> where T: Clone {
        self.0.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::OverwriteRB;

    #[test]
    fn test_overwrite_rb() {
        let mut rb: OverwriteRB<i32, 3> = OverwriteRB::new();
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());

        rb.push(1);
        rb.push(2);
        assert_eq!(rb.len(), 2);
        assert!(!rb.is_empty());
        assert_eq!(rb.to_vec(), vec![1, 2]);

        rb.push(3);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.to_vec(), vec![1, 2, 3]);

        rb.push(4);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.to_vec(), vec![2, 3, 4]);

        let mut rb2: OverwriteRB<i32, 3> = OverwriteRB::new();
        rb2.push_many(vec![1, 2, 3, 4, 5].into_iter());
        assert_eq!(rb2.len(), 3);
        assert_eq!(rb2.to_vec(), vec![3, 4, 5]);
    }
}
