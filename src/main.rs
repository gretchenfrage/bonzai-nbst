
#![feature(nll)]

extern crate bonzai;
extern crate rand;

mod bst;

use bst::Bst;

use std::collections::HashSet;

use rand::prelude::*;
use rand::XorShiftRng;


pub fn cross_check<A: Bst<i32>, B: Bst<i32>>()
    where for<'s> &'s A: IntoIterator<Item = &'s i32> ,
          for<'s> &'s B: IntoIterator<Item = &'s i32> {

    let seed = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut rng = XorShiftRng::from_seed(seed);

    let mut a = A::new();
    let mut b = B::new();

    for i in 0..10000 {
        match rng.gen::<u8>() % 4 {
            0 | 1 => {
                // insert random element
                let n: i32 = rng.gen();
                a.insert(n);
                b.insert(n);
            },
            2 => {
                // remove random element
                let n: i32 = rng.gen();
                a.remove(&n);
                b.remove(&n);
            },
            3 => {
                // check consistency
                let mut a_size = 0;
                for n in &a {
                    assert!(a.contains(n));
                    assert!(b.contains(n));
                    a_size += 1;
                }
                let mut b_size = 0;
                for n in &b {
                    assert!(a.contains(n));
                    assert!(b.contains(n));
                    b_size += 1;
                }
                assert_eq!(a_size, b_size);
                println!("i={}, all consistent", i);
            },
            _ => unreachable!()
        };
    }
}


fn main() {
    // self consistency check
        /*
    cross_check::<
        bst::stdlib::BTreeSet<i32>,
        bst::stdlib::BTreeSet<i32>
    >();
    */

    // correctness check
    cross_check::<
        bst::stdlib::BTreeSet<i32>,
        bst::bonzai::BonzaiBst<i32>,
    >();
}