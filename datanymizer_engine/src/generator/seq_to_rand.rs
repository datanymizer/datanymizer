use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

type RandNumber = f64;

pub trait SeqToRand {
    fn rand_for(&self, n: usize) -> RandNumber;
}

pub struct HashSeqToRand {
    state: RandomState,
}

impl HashSeqToRand {
    pub fn new() -> Self {
        let state = RandomState::new();
        Self { state }
    }
}

impl SeqToRand for HashSeqToRand {
    fn rand_for(&self, n: usize) -> RandNumber {
        let mut hasher = self.state.build_hasher();
        hasher.write_usize(n);
        hasher.finish() as RandNumber / usize::MAX as RandNumber
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rand_for() {
        let r = HashSeqToRand::new();
        let other_r = HashSeqToRand::new();

        let res5 = r.rand_for(5);
        let res10 = r.rand_for(10);

        assert_ne!(res5, res10);
        assert_eq!(res5, r.rand_for(5));
        assert_eq!(res5, r.rand_for(5));
        assert_eq!(res10, r.rand_for(10));
        assert_eq!(res10, r.rand_for(10));

        let other_res5 = other_r.rand_for(5);
        let other_res10 = other_r.rand_for(10);

        assert_ne!(res5, other_res5);
        assert_ne!(res10, other_res10);
        assert_ne!(other_res5, other_res10);
        assert_eq!(other_res5, other_r.rand_for(5));
        assert_eq!(other_res5, other_r.rand_for(5));
        assert_eq!(other_res10, other_r.rand_for(10));
        assert_eq!(other_res10, other_r.rand_for(10));
    }
}
