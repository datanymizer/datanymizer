pub trait Key {
    fn len(&self) -> usize;

    fn index(&self, i: usize) -> usize;

    fn iter(&self) -> KeyIter<Self>
    where
        Self: Sized,
    {
        KeyIter::new(self)
    }
}

pub struct MonotonicKey {
    start: usize,
    len: usize,
}

impl MonotonicKey {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn from_one(len: usize) -> Self {
        Self::new(1, len)
    }
}

impl Key for MonotonicKey {
    fn len(&self) -> usize {
        self.len
    }

    fn index(&self, i: usize) -> usize {
        if i < self.len {
            self.start + i
        } else {
            panic!("Index is out of bounds")
        }
    }
}

pub struct KeyIter<'a, K: Key> {
    i: usize,
    len: usize,
    key: &'a K,
}

impl<'a, K: Key> KeyIter<'a, K> {
    pub fn new(key: &'a K) -> Self {
        let i = 0;
        let len = key.len();
        Self { i, len, key }
    }
}

impl<K: Key> Iterator for KeyIter<'_, K> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.len {
            self.i += 1;
            Some(self.key.index(self.i - 1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter() {
        let k = MonotonicKey::from_one(4);
        let mut iter = k.iter();
        for i in vec![1, 2, 3, 4] {
            assert_eq!(iter.next(), Some(i));
        }
        assert_eq!(iter.next(), None);
    }
}
