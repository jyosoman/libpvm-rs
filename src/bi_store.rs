use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::collections::hash_set::Iter;
use std::hash::Hash;
use std::cmp::Eq;
use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct BiStore<T, U>
where
    T: Eq + Hash + Copy + Debug,
    U: Eq + Hash + Copy + Debug,
{
    fwd: HashMap<T, HashSet<U>>,
    rev: HashMap<U, HashSet<T>>,
}

impl<T, U> BiStore<T, U>
    where
        T: Eq + Hash + Copy + Debug,
        U: Eq + Hash + Copy + Debug,
{
    fn insert(&mut self, a: T, b: U) -> bool {
        if self.rev.contains_key(&b) {
            self.rev.get_mut(&b).unwrap().insert(a);
        } else {
            self.rev.insert(b, hashset!(a));
        }
        if self.fwd.contains_key(&a) {
            self.fwd.get_mut(&a).unwrap().insert(b)
        } else {
            self.fwd.insert(a, hashset!(b));
            true
        }
    }

    fn remove(&mut self, a:T, b:U) -> bool {
        if self.rev.contains_key(&b) {
            self.rev.get_mut(&b).unwrap().remove(&a);
        }
        if self.fwd.contains_key(&a) {
            self.fwd.get_mut(&a).unwrap().remove(&b)
        } else {
            false
        }
    }

    fn is_empty(&self, a:T) -> bool {
        if self.fwd.contains_key(&a) {
            self.fwd[&a].is_empty()
        } else {
            true
        }
    }

    fn rev_iter(&self, b:U) -> Iter<T>{
        self.rev[&b].iter()
    }
}


#[cfg(test)]
mod tests {
    use super::BiStore;

    #[test]
    fn regular_use() {
        let mut s : BiStore<i64, i64> = Default::default();
        println!("{:?}", &s);
        assert!(s.insert(1,1));
        println!("{:?}", &s);
        assert!(s.insert(1,2));
        println!("{:?}", &s);
        assert!(!s.insert(1,2));
        println!("{:?}", &s);
        let mut i = s.rev_iter(2);
        assert_eq!(i.next(), Some(&1));
        assert_eq!(i.next(), None);
    }
}