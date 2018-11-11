
use super::Bst;

pub use std::collections::BTreeSet;
use std::fmt::Debug;

impl<T: Ord + Debug> Bst<T> for BTreeSet<T> {
    fn new() -> Self {
        BTreeSet::new()
    }

    fn insert(&mut self, elem: T) -> bool {
        self.insert(elem)
    }

    fn remove(&mut self, elem: &T) -> bool {
        self.remove(elem)
    }

    fn contains(&self, elem: &T) -> bool {
        self.contains(elem)
    }
}