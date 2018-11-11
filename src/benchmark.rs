
use bst::Bst;

use std::collections::BTreeSet;
use std::fmt::Debug;

use rand::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::XorShiftRng;
use stopwatch::Stopwatch;

#[derive(Debug, Copy, Clone)]
pub enum Op<T> {
    Insert(T),
    Remove(T),
    Contains(T),
}

pub fn first_after<T: Clone + Ord>(set: &BTreeSet<T>, elem: T) -> Option<T> {
    set.range(..elem).next_back().cloned()
}

pub fn rand_ops<T: Clone + Ord>(num_ops: usize) -> Vec<Op<T>> where Standard: Distribution<T> {
    //let mut rng = thread_rng();
    let seed = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut rng = XorShiftRng::from_seed(seed);

    let mut contains = BTreeSet::new();

    let mut ops = Vec::new();
    for _ in 0..num_ops {
        match rng.gen::<u8>() % 7 {
            0 | 6 => {
                // insert rand elem
                let t: T = rng.gen();
                ops.push(Op::Insert(t.clone()));
                contains.insert(t);
            },
            1 => {
                // remove rand elem
                let t: T = rng.gen();
                ops.push(Op::Remove(t.clone()));
                contains.remove(&t);
            },
            2 => {
                // check contains rand elem
                let t: T = rng.gen();
                ops.push(Op::Contains(t));
            }
            3 => {
                // insert existent elem
                if contains.len() == 0 {
                    continue;
                }
                let t: Option<T> = first_after(&contains, rng.gen::<T>());
                if let Some(t) = t {
                    ops.push(Op::Insert(t.clone()));
                    contains.insert(t);
                }
            },
            4 => {
                // remove existent elem
                if contains.len() == 0 {
                    continue;
                }
                let t: Option<T> = first_after(&contains, rng.gen::<T>());
                if let Some(t) = t {
                    ops.push(Op::Remove(t.clone()));
                    contains.remove(&t);
                }
            },
            5 => {
                // check contains existent elem
                if contains.len() == 0 {
                    continue;
                }
                let t: Option<T> = first_after(&contains, rng.gen::<T>());
                if let Some(t) = t {
                    ops.push(Op::Insert(t.clone()));
                    ops.push(Op::Contains(t));
                }
            },
            _ => unreachable!()
        };
    }

    ops
}

pub fn time_ms<T: Ord + Debug, B: Bst<T>>(ops: Vec<Op<T>>) -> f64
    where for<'s> &'s B: IntoIterator<Item = &'s T>{

    let timer = Stopwatch::start_new();
    let mut tree = B::new();
    for op in ops {
        match op {
            Op::Insert(t) => {
                tree.insert(t);
            },
            Op::Remove(t) => {
                tree.remove(&t);
            },
            Op::Contains(t) => {
                tree.contains(&t);
            }
        };
    }
    timer.elapsed().as_millis() as f64
}