
pub mod bonzai;
pub mod stdlib;

use std::iter::IntoIterator;
use std::fmt::Debug;

pub trait Bst<T: Ord + Debug>: Debug
    where for<'s> &'s Self: IntoIterator<Item = &'s T> {
    fn new() -> Self;

    fn insert(&mut self, elem: T) -> bool;

    fn remove(&mut self, elem: &T) -> bool;

    fn contains(&self, elem: &T) -> bool;
}