use std::sync::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::Cell;

const N: usize = 256;

#[derive(Default)]
pub struct InvBloom {
    data: Vec<Mutex<Cell<String>>>,
}

impl InvBloom {
    pub fn new() -> InvBloom {
        let mut data = Vec::with_capacity(N);
        for _ in 0..N {
            data.push(Mutex::new(Cell::new(String::from(""))));
        }
        InvBloom { data: data }
    }

    pub fn check(&self, test: &String) -> bool {
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
