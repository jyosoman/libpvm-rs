use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::cmp::Eq;
use std::thread;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

enum StoreState<V> {
    Present(V),
    Loaned,
    AwaitingDrop,
}

use self::StoreState::{AwaitingDrop, Loaned, Present};

#[derive(Default)]
pub struct CheckingStore<K, V>
where
    K: Hash + Eq + Copy,
{
    store: HashMap<K, StoreState<V>>,
    outstanding: AtomicUsize,
}

impl<K, V> CheckingStore<K, V>
where
    K: Hash + Eq + Copy,
{
    pub fn new() -> CheckingStore<K, V> {
        CheckingStore {
            store: HashMap::new(),
            outstanding: AtomicUsize::new(0),
        }
    }
    pub fn contains_key(&self, key: K) -> bool {
        match self.store.get(&key) {
            Some(v) => match *v {
                Present(_) | Loaned => true,
                AwaitingDrop => false,
            },
            None => false,
        }
    }
    pub fn insert(&mut self, key: K, val: V) {
        match self.store.entry(key) {
            Entry::Vacant(e) => {
                e.insert(Present(val));
            }
            Entry::Occupied(mut e) => {
                let prev = e.insert(Present(val));
                match prev {
                    Present(_) => {}
                    Loaned => panic!("Cannot overwrite loaned value"),
                    AwaitingDrop => panic!("Cannot overwrite value awaiting drop"),
                }
            }
        }
    }
    pub fn remove(&mut self, key: K) -> bool {
        match self.store.entry(key) {
            Entry::Occupied(mut e) => {
                let v = e.insert(AwaitingDrop);
                match v {
                    Present(_) => {
                        e.remove();
                        true
                    }
                    Loaned => true,
                    AwaitingDrop => false,
                }
            }
            Entry::Vacant(_) => false,
        }
    }
    pub fn checkout(&mut self, key: K) -> Option<DropGuard<K, V>> {
        let ptr: *mut Self = self;
        match self.store.entry(key) {
            Entry::Occupied(mut e) => {
                let v = e.insert(Loaned);
                match v {
                    Present(val) => {
                        self.outstanding.fetch_add(1, Ordering::SeqCst);
                        Some(DropGuard {
                            owner: ptr,
                            key: Some(key),
                            inner: Some(val),
                        })
                    }
                    Loaned => panic!("Checking out already checked out value"),
                    AwaitingDrop => panic!("Checking out value awaiting drop"),
                }
            }
            Entry::Vacant(_) => None,
        }
    }
    fn checkin(&mut self, key: K, val: V) {
        match self.store.entry(key) {
            Entry::Occupied(mut e) => {
                self.outstanding.fetch_sub(1, Ordering::SeqCst);
                let v = e.insert(Present(val));
                match v {
                    Present(_) => panic!("Returning replaced item"),
                    Loaned => {}
                    AwaitingDrop => {
                        e.remove();
                    }
                }
            }
            Entry::Vacant(_) => panic!("Checking in item not from store"),
        }
    }
}

impl<K, V> Drop for CheckingStore<K, V>
where
    K: Hash + Eq + Copy,
{
    fn drop(&mut self) {
        if !thread::panicking() {
            let count = self.outstanding.load(Ordering::SeqCst);
            if count != 0 {
                panic!("{} value checkouts outlived store.", count)
            }
        }
    }
}

pub struct DropGuard<K, V>
where
    K: Hash + Eq + Copy,
{
    owner: *mut CheckingStore<K, V>,
    key: Option<K>,
    inner: Option<V>,
}

impl<K, V> Drop for DropGuard<K, V>
where
    K: Hash + Eq + Copy,
{
    fn drop(&mut self) {
        if self.inner.is_some() && !thread::panicking() {
            unsafe {
                (*self.owner).checkin(self.key.take().unwrap(), self.inner.take().unwrap());
            }
        }
    }
}

impl<K, V> Deref for DropGuard<K, V>
where
    K: Hash + Eq + Copy,
{
    type Target = V;

    fn deref(&self) -> &V {
        self.inner.as_ref().unwrap()
    }
}

impl<K, V> DerefMut for DropGuard<K, V>
where
    K: Hash + Eq + Copy,
{
    fn deref_mut(&mut self) -> &mut V {
        self.inner.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::CheckingStore;
    use super::DropGuard;
    use super::Ordering;
    #[test]
    fn basic_use() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        assert_eq!(s.outstanding.load(Ordering::SeqCst), 0);
        {
            s.insert(1, String::from("test"));
            assert!(s.contains_key(1));
            s.insert(2, String::from("double test"));
            assert_eq!(s.outstanding.load(Ordering::SeqCst), 0);
            {
                let mut first = s.checkout(1).unwrap();
                assert_eq!(s.outstanding.load(Ordering::SeqCst), 1);
                s.insert(3, String::from("even more test"));
                assert_eq!(*first, "test");
                first.push_str("-even more");
                assert_eq!(*first, "test-even more");
            }
            assert_eq!(s.outstanding.load(Ordering::SeqCst), 0);
            let first = s.checkout(1).unwrap();
            assert_eq!(s.outstanding.load(Ordering::SeqCst), 1);
            assert_eq!(*first, "test-even more");
            s.insert(2, String::from("insert test"));
            assert!(s.remove(2));
            assert!(!s.contains_key(2));
        }
        assert_eq!(s.outstanding.load(Ordering::SeqCst), 0);
    }

    #[test]
    #[should_panic(expected = "1 value checkouts outlived store.")]
    fn failure_to_return() {
        {
            let mut s: CheckingStore<i64, String> = CheckingStore::new();
            s.insert(1, String::from("test"));
            let _v = s.checkout(1).unwrap();
            drop(s);
        }
    }

    #[test]
    #[should_panic(expected = "Returning replaced item")]
    fn double_reinsert() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        {
            let _v = s.checkout(1);
            let _v2 = DropGuard {
                owner: &mut s as *mut CheckingStore<i64, String>,
                key: Some(1),
                inner: Some(String::from("test")),
            };
        }
    }

    #[test]
    #[should_panic(expected = "Checking in item not from store")]
    fn returning_none_store() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        {
            let _v = DropGuard {
                owner: &mut s as *mut CheckingStore<i64, String>,
                key: Some(1),
                inner: Some(String::from("boo")),
            };
        }
    }

    #[test]
    #[should_panic(expected = "Checking out already checked out value")]
    fn double_checkout() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let _a = s.checkout(1).unwrap();
        let _b = s.checkout(1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Checking out value awaiting drop")]
    fn double_checkout_drop() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let _a = s.checkout(1).unwrap();
        s.remove(1);
        let _b = s.checkout(1).unwrap();
    }

    #[test]
    fn remove_indempotent() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let _a = s.checkout(1).unwrap();
        assert!(s.remove(1));
        for _ in 0..100 {
            assert!(!s.remove(1));
        }
    }

    #[test]
    fn double_insert() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        s.insert(1, String::from("test"));
    }

    #[test]
    #[should_panic(expected = "Cannot overwrite loaned value")]
    fn double_insert_loaned() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let _v = s.checkout(1);
        s.insert(1, String::from("test"));
    }

    #[test]
    #[should_panic(expected = "Cannot overwrite value awaiting drop")]
    fn double_insert_drop() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let _v = s.checkout(1);
        s.remove(1);
        s.insert(1, String::from("test"));
    }
}
