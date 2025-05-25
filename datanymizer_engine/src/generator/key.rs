pub trait Key {
    fn len(&self) -> usize;

    fn index(&self, i: usize) -> usize;
}

impl Key for Box<dyn Key> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn index(&self, i: usize) -> usize {
        self.as_ref().index(i)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index() {
        let k = MonotonicKey::from_one(4);
        for (i, v) in [1, 2, 3, 4].iter().enumerate() {
            assert_eq!(k.index(i), *v);
        }
    }
}
