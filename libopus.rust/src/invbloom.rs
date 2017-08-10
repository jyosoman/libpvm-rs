/*
 * Reverse bloom filter based on
 * https://www.somethingsimilar.com/2012/05/21/the-opposite-of-a-bloom-filter/
 */

use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use trace::Uuid5;

const N: usize = 256;  // have to pick power of 2
const NMASK: usize = N - 1;

#[derive(Default)]
pub struct InvBloom {
    data: Vec<AtomicUsize>,
}

impl InvBloom {
    pub fn new() -> InvBloom {
        let mut data = Vec::with_capacity(N);
        for _ in 0..N {
            data.push(AtomicUsize::new(0));
        }
        InvBloom { data: data }
    }

    pub fn check(&self, test: &Uuid5) -> bool {
        let hash = {
            let mut hasher = DefaultHasher::new();
            test.hash(&mut hasher);
            hasher.finish() as usize
        };
        let prev = self.data[hash & NMASK].swap(hash, Ordering::Relaxed);
        prev == hash
    }
}
