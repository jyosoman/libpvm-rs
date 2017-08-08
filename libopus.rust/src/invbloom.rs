use std::sync::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::Cell;
use trace::uuid5;

const N: usize = 256;

#[derive(Default)]
pub struct InvBloom {
    data: Vec<Mutex<Cell<uuid5>>>,
}

impl InvBloom {
    pub fn new() -> InvBloom {
        let mut data = Vec::with_capacity(N);
        for _ in 0..N {
            data.push(Mutex::new(Cell::new(uuid5::zero())));
        }
        InvBloom { data: data }
    }

    pub fn check(&self, test: &uuid5) -> bool {
        let hash = {
            let mut hasher = DefaultHasher::new();
            test.hash(&mut hasher);
            (hasher.finish() as usize) % N
        };
        let didx = self.data[hash].lock().unwrap();
        let prev = didx.replace(test.clone());
        &prev == test
    }
}
