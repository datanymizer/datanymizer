use super::{
    key::Key,
    seq_to_rand::{HashSeqToRand, SeqToRand},
};

pub trait ForeignKey {
    type Src: Key;

    fn source(&self) -> &Self::Src;

    fn source_index(&self, i: usize) -> usize;

    fn len(&self) -> usize;
}

impl<FK> Key for FK
where
    FK: ForeignKey,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn index(&self, i: usize) -> usize {
        self.source().index(self.source_index(i))
    }
}

pub struct MonotonicFKey<'a, Src: Key> {
    src: &'a Src,
    len: usize,
}

impl<'a, Src: Key> MonotonicFKey<'a, Src> {
    pub fn new(src: &'a Src, len: usize) -> Self {
        Self { src, len }
    }
}

impl<Src: Key> ForeignKey for MonotonicFKey<'_, Src> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src
    }

    fn source_index(&self, i: usize) -> usize {
        ((i as f64 / self.len as f64) * self.src.len() as f64).floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub struct RandomFKey<'a, Src: Key, StR: SeqToRand> {
    src: &'a Src,
    len: usize,
    seq_to_rand: StR,
}

impl<'a, Src: Key, StR: SeqToRand> RandomFKey<'a, Src, StR> {
    pub fn new(src: &'a Src, len: usize, seq_to_rand: StR) -> Self {
        Self {
            src,
            len,
            seq_to_rand,
        }
    }
}

impl<'a, Src: Key, StR: SeqToRand> ForeignKey for RandomFKey<'a, Src, StR> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src
    }

    fn source_index(&self, i: usize) -> usize {
        (self.src.len() as f64 * self.seq_to_rand.rand_for(i)).floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub struct MonotonicRandomFKey<'a, Src: Key, StR: SeqToRand> {
    src: &'a Src,
    len: usize,
    seq_to_rand: StR,
}

impl<'a, Src: Key, StR: SeqToRand> MonotonicRandomFKey<'a, Src, StR> {
    pub fn new(src: &'a Src, len: usize, seq_to_rand: StR) -> Self {
        Self {
            src,
            len,
            seq_to_rand,
        }
    }
}

impl<'a, Src: Key, StR: SeqToRand> ForeignKey for MonotonicRandomFKey<'a, Src, StR> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src
    }

    fn source_index(&self, i: usize) -> usize {
        ((i as f64 / self.len as f64) * self.src.len() as f64 * self.seq_to_rand.rand_for(i))
            .floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

fn default_seq_to_rand() -> HashSeqToRand {
    HashSeqToRand::new()
}

#[cfg(test)]
mod test {
    use super::super::key::MonotonicKey;
    use super::*;

    #[test]
    fn iter() {
        let k = MonotonicKey::from_one(2);
        let fk = MonotonicFKey::new(&k, 6);
        let mut iter = fk.iter();
        for i in vec![1, 1, 1, 2, 2, 2] {
            assert_eq!(iter.next(), Some(i));
        }
        assert_eq!(iter.next(), None);

        let fk2 = MonotonicFKey::new(&fk, 9);
        let mut iter = fk2.iter();
        for i in vec![1, 1, 1, 1, 1, 2, 2, 2, 2] {
            assert_eq!(iter.next(), Some(i));
        }
    }
}
