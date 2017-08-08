/*
 * Reverse bloom filter based on
 * https://www.somethingsimilar.com/2012/05/21/the-opposite-of-a-bloom-filter/
 */

use std::sync::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::Cell;

const N: usize = 256;

#[derive(Default)]
pub struct InvBloom {
    data: Vec<Mutex<Cell<usize>>>,
}

impl InvBloom {
    pub fn new() -> InvBloom {
        let mut data = Vec::with_capacity(N);
        for _ in 0..N {
            data.push(Mutex::new(Cell::new(0)));
        }
        InvBloom { data: data }
    }

    pub fn check(&self, test: &String) -> bool {
        let hash = {
            let mut hasher = DefaultHasher::new();
            test.hash(&mut hasher);
            hasher.finish() as usize
        };
        let prev = self.data[hash % N].lock().unwrap().replace(hash);
        prev == hash
    }
}
