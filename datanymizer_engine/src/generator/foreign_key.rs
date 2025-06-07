use super::{
    key::Key,
    seq_to_rand::{HashSeqToRand, SeqToRand},
};
use std::sync::Arc;

pub trait ForeignKey: Send + Sync {
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

pub struct MonotonicFKey<Src: Key> {
    src: Arc<Src>,
    len: usize,
}

impl<Src: Key> MonotonicFKey<Src> {
    pub fn new(src: Arc<Src>, len: usize) -> Self {
        Self { src, len }
    }
}

impl<Src: Key> ForeignKey for MonotonicFKey<Src> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src.as_ref()
    }

    fn source_index(&self, i: usize) -> usize {
        ((i as f64 / self.len as f64) * self.src.len() as f64).floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub struct RandomFKey<Src: Key, StR: SeqToRand> {
    src: Arc<Src>,
    len: usize,
    seq_to_rand: StR,
}

impl<Src: Key, StR: SeqToRand> RandomFKey<Src, StR> {
    pub fn new(src: Arc<Src>, len: usize, seq_to_rand: StR) -> Self {
        Self {
            src,
            len,
            seq_to_rand,
        }
    }
}

impl<Src: Key, StR: SeqToRand> ForeignKey for RandomFKey<Src, StR> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src.as_ref()
    }

    fn source_index(&self, i: usize) -> usize {
        (self.src.len() as f64 * self.seq_to_rand.rand_for(i)).floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub struct MonotonicRandomFKey<Src: Key, StR: SeqToRand> {
    src: Arc<Src>,
    len: usize,
    seq_to_rand: StR,
}

impl<Src: Key, StR: SeqToRand> MonotonicRandomFKey<Src, StR> {
    pub fn new(src: Arc<Src>, len: usize, seq_to_rand: StR) -> Self {
        Self {
            src,
            len,
            seq_to_rand,
        }
    }
}

impl<Src: Key, StR: SeqToRand> ForeignKey for MonotonicRandomFKey<Src, StR> {
    type Src = Src;

    fn source(&self) -> &Self::Src {
        self.src.as_ref()
    }

    fn source_index(&self, i: usize) -> usize {
        ((i as f64 / self.len as f64) * self.src.len() as f64 * self.seq_to_rand.rand_for(i))
            .floor() as usize
    }

    fn len(&self) -> usize {
        self.len
    }
}

pub fn default_seq_to_rand() -> HashSeqToRand {
    HashSeqToRand::new()
}

#[cfg(test)]
mod test {
    use super::super::key::MonotonicKey;
    use super::*;

    #[test]
    fn index() {
        let k = MonotonicKey::new(1, 2);
        let fk = MonotonicFKey::new(Arc::new(k), 6);
        for (i, v) in [1, 1, 1, 2, 2, 2].into_iter().enumerate() {
            assert_eq!(fk.index(i), v);
        }

        let fk2 = MonotonicFKey::new(Arc::new(fk), 9);
        for (i, v) in [1, 1, 1, 1, 1, 2, 2, 2, 2].into_iter().enumerate() {
            assert_eq!(fk2.index(i), v);
        }
    }
}
