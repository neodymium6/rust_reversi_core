use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct StackVec64<T> {
    data: [T; 64],
    len: usize,
}

impl<T: Copy + Default> StackVec64<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: [T::default(); 64],
            len: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        debug_assert!(self.len < 64, "StackVec64 is full");
        self.data[self.len] = value;
        self.len += 1;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            vec: self,
            index: 0,
        }
    }

    #[inline]
    pub fn sort_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, f: F) {
        self.data[..self.len].sort_by_key(f)
    }

    #[inline]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|y| x == y)
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().copied().collect()
    }
}

impl<T: Copy + Default> Default for StackVec64<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for StackVec64<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "index out of bounds");
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for StackVec64<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "index out of bounds");
        &mut self.data[index]
    }
}

pub struct Iter<'a, T> {
    vec: &'a StackVec64<T>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let item = &self.vec.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T: Copy + Default> IntoIterator for &'a StackVec64<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
