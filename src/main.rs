
#![feature(nll)]
#![feature(duration_as_u128)]

extern crate bonzai;
extern crate rand;
extern crate stopwatch;

mod bst;
mod benchmark;

use bst::Bst;

use std::collections::HashSet;
use std::env::args;

use rand::prelude::*;
use rand::XorShiftRng;


pub fn cross_check<A: Bst<i32>, B: Bst<i32>>()
    where for<'s> &'s A: IntoIterator<Item = &'s i32> ,
          for<'s> &'s B: IntoIterator<Item = &'s i32> {

    let seed = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let mut rng = XorShiftRng::from_seed(seed);

    let mut a = A::new();
    let mut b = B::new();
    let mut h = HashSet::new();

    for i in 0..10000 {
        match rng.gen::<u8>() % 6 {
            0 | 1 => {
                // insert random element
                let n: i32 = rng.gen::<i32>() % 1000;
                a.insert(n);
                b.insert(n);
                h.insert(n);
            },
            2 => {
                // remove random element
                let n: i32 = rng.gen::<i32>() % 1000;
                a.remove(&n);
                b.remove(&n);
                h.remove(&n);
            },
            3 => {
                // check consistency
                let mut a_size = 0;
                for n in &a {
                    assert!(a.contains(n));
                    if !b.contains(n) {
                        eprintln!("b doesn't contain {}", n);
                        eprintln!("{:#?}", b);
                        return;
                    }
                    a_size += 1;
                }
                let mut b_size = 0;
                for n in &b {
                    assert!(a.contains(n));
                    assert!(b.contains(n));
                    b_size += 1;
                }
                assert_eq!(a_size, b_size);
                let h_size = h.len();
                assert_eq!(a_size, h_size);
                assert_eq!(b_size, h_size);
                println!("i={}, all consistent", i);
            },
            4 => {
                // remove random pre-existent element
                let n: Option<i32> = h.iter().cloned().nth(rng.gen::<usize>() & h.len());
                if let Some(n) = n {
                    a.remove(&n);
                    b.remove(&n);
                    h.remove(&n);
                }
            },
            5 => {
                // insert random pre-existent element
                let n: Option<i32> = h.iter().cloned().nth(rng.gen::<usize>() & h.len());
                if let Some(n) = n {
                    a.insert(n);
                    b.insert(n);
                    h.insert(n);
                }
            }
            _ => unreachable!()
        };
    }
}



fn main() {
    match args().collect::<Vec<String>>().as_slice() {
        &[_, ref num_ops] => {
            let num_ops: usize = num_ops.parse()
                .expect("invalid num ops");

            let ops = benchmark::rand_ops::<i32>(num_ops);

            let bonzai_ms = benchmark::time_ms::<i32, bst::bonzai::BonzaiBst<i32>>(ops.clone());
            println!("bonzai ms:");
            eprintln!("{}", bonzai_ms);
            let boxy_ms = benchmark::time_ms::<i32, bst::boxy::BoxBst<i32>>(ops.clone());
            println!("boxy ms:");
            eprintln!("{}", boxy_ms);
        },
        _ => {
            eprintln!("use: ./bonzai-nbst [num_ops]")
        }
    }
}