use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::thread;
use std::ops::{Deref, DerefMut};

pub struct CheckingStore<K, V>
    where
        K: Hash + Eq + Clone,
{
    store: HashMap<K, Option<V>>,
}

impl<K, V> CheckingStore<K, V>
    where
        K: Hash + Eq + Clone,
{
    pub fn new() -> CheckingStore<K, V> {
        CheckingStore{
            store: HashMap::new()
        }
    }
    pub fn contains_key(&self, key: &K) -> bool {
        self.store.contains_key(key)
    }
    pub fn insert(&mut self, key: K, val: V) {
        if self.store.contains_key(&key) {
            panic!("Cannot overwrite store entry");
        }
        self.store.insert(key, Some(val));
    }
    pub fn remove(&mut self, key: &K) {
        self.store.remove(key);
    }
    pub fn checkout(&mut self, key: &K) -> Option<DropGuard<K, V>> {
        match self.store.get_mut(key) {
            Some(v) => Some(DropGuard::new(
                key.clone(),
                v.take().expect("Checking out already checked out value"),
            )),
            None => None,
        }
    }
    pub fn checkin(&mut self, guard: DropGuard<K, V>) {
        let (key, val) = DropGuard::unwrap(guard);
        if !self.store.contains_key(&key) {
            panic!("Returning item not borrowed from store");
        }
        if self.store[&key].is_some() {
            panic!("Returning replaced item");
        }
        self.store.insert(key, Some(val));
    }
}

pub struct DropGuard<K, V> {
    key: Option<K>,
    inner: Option<V>,
}

impl<K, V> DropGuard<K, V> {
    fn new(key: K, val: V) -> DropGuard<K, V> {
        DropGuard {
            key: Some(key),
            inner: Some(val),
        }
    }

    fn unwrap(mut guard: DropGuard<K, V>) -> (K, V) {
        (guard.key.take().unwrap(), guard.inner.take().unwrap())
    }
}

impl<K, V> Drop for DropGuard<K, V> {
    fn drop(&mut self) {
        if self.inner.is_some() && !thread::panicking() {
            panic!("Object not returned to store.");
        }
    }
}

impl<K, V> Deref for DropGuard<K, V> {
    type Target = V;

    fn deref(&self) -> &V {
        self.inner.as_ref().unwrap()
    }
}

impl<K, V> DerefMut for DropGuard<K, V> {
    fn deref_mut(&mut self) -> &mut V {
        self.inner.as_mut().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::CheckingStore;
    use super::DropGuard;
    #[test]
    fn basic_use() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        assert!(s.contains_key(&1));
        s.insert(2, String::from("double test"));
        {
            let mut first = s.checkout(&1).unwrap();
            s.insert(3, String::from("even more test"));
            assert_eq!(*first, "test");
            first.push_str("-even more");
            assert_eq!(*first, "test-even more");
            s.checkin(first);
        }
        let first = s.checkout(&1).unwrap();
        assert_eq!(*first, "test-even more");
        s.checkin(first);

        assert!(s.contains_key(&2));
        s.remove(&2);
        assert!(!s.contains_key(&2));
    }

    #[test]
    #[should_panic(expected = "Object not returned to store.")]
    fn failure_to_return() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        s.checkout(&1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Returning replaced item")]
    fn double_reinsert() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let first = s.checkout(&1).unwrap();
        s.checkin(DropGuard::new(1, String::from("boo")));
        s.checkin(first);
    }

    #[test]
    #[should_panic(expected = "Returning item not borrowed from store")]
    fn returning_none_store() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.checkin(DropGuard::new(1, String::from("boo")))
    }

    #[test]
    #[should_panic(expected = "Checking out already checked out value")]
    fn double_checkout(){
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        let a = s.checkout(&1).unwrap();
        let b = s.checkout(&1).unwrap();
        s.checkin(a);
        s.checkin(b);
    }

    #[test]
    #[should_panic(expected = "Cannot overwrite store entry")]
    fn double_insert() {
        let mut s: CheckingStore<i64, String> = CheckingStore::new();
        s.insert(1, String::from("test"));
        s.insert(1, String::from("test"));
    }
}