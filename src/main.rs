//! Generates statistics for different types of tables
//!
//! Use by changing the constants found throughout the program
//! Variants tested can be adjusted by changing the vec `tables` in main
//! load factors tested can be adjusted by changing the constant `LOAD_FACTORS`

extern crate gnuplot;
extern crate rand;

pub mod hashset;
pub mod logging;

use hashset::*;
use logging::*;
use rand::{thread_rng, Rng};
use std::time::Instant;

/// Helper function to get an instance of a DefaultHashTableBuilder for the given HashTable
fn get_builder<T: PartialEq + 'static, H: 'static + HashTable<T> + Default>(
) -> Box<dyn HashTableBuilder<T>> {
    Box::new(DefaultHashTableBuilder::<T, H>::default())
}

/// Resizes every type of HashTable, so they take up
/// nearly the same space in memory
const RESIZE_TO_MAKE_FAIR: bool = true;
/// How many elements to insert into the HashTable before doing
/// probing tests
const LOAD_FACTORS: [f64; 32] = [
    0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.1, 0.11, 0.12, 0.13, 0.14, 0.15, 0.16,
    0.17, 0.18, 0.19, 0.2, 0.21, 0.22, 0.23, 0.24, 0.25, 0.26, 0.27, 0.28, 0.29, 0.3, 0.31, 0.32,
];
/// How many tests to do at each load factor
const ITERATIONS_PER_LOAD_FACTOR: usize = 50;

fn main() {
    // All variants of HashTable possible in this module
    #[rustfmt::skip]
    let tables: Vec<(Box<dyn HashTableBuilder<u32>>, String)> = vec![
        (get_builder::<u32, OpenAddressingTable::<u32, QuadraticProber, MulHash>>(), "Quadratic Mul".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, QuadraticProber, ModHash>>(), "Quadratic Mod".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, QuadraticProber, XorShiftHash>>(), "Quadratic XOR".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, LinearProber, MulHash>>(), "Linear Mul".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, LinearProber, ModHash>>(), "Linear Mod".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, LinearProber, XorShiftHash>>(), "Linear XOR".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, TriangularProber, MulHash>>(), "Triangular Mul".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, TriangularProber, ModHash>>(), "Triangular Mod".to_owned()),
        (get_builder::<u32, OpenAddressingTable::<u32, TriangularProber, XorShiftHash>>(), "Triangular XOR".to_owned()),
        (get_builder::<u32, DirectChainingTable::<u32, MulHash>>(), "Direct Mul".to_owned()),
        (get_builder::<u32, DirectChainingTable::<u32, ModHash>>(), "Direct Mod".to_owned()),
        (get_builder::<u32, DirectChainingTable::<u32, XorShiftHash>>(), "Direct XOR".to_owned()),
        (get_builder::<u32, SeparateChainingTable::<u32, MulHash>>(), "Separate Mul".to_owned()),
        (get_builder::<u32, SeparateChainingTable::<u32, ModHash>>(), "Separate Mod".to_owned()),
        (get_builder::<u32, SeparateChainingTable::<u32, XorShiftHash>>(), "Separate XOR".to_owned()),
        (get_builder::<u32, CoalescedTable::<u32, MulHash>>(), "Coalesced Mul".to_owned()),
        (get_builder::<u32, CoalescedTable::<u32, ModHash>>(), "Coalesced Mod".to_owned()),
        (get_builder::<u32, CoalescedTable::<u32, XorShiftHash>>(), "Coalesced XOR".to_owned()),
    ];
    generate_stats(tables);
}

/// generates and outputs stats
///
/// generates and outputs stats for every table in `tables`.
/// Stats are: How many collisions on successful find, how many collisions on
/// failed find, how much time on successful find, how much time on failed find;
/// Stats are output to stdout, a hardcoded csv file and 4 graphs (one for every stat)
/// Stats are calculated at every load_factor in LOAD_FACTORS
/// ITERATIONS_PER_LOAD_FACTOR times
fn generate_stats(tables: Vec<(Box<dyn HashTableBuilder<u32>>, String)>) {
    let mut all_stats = Vec::new();

    for (builder, name) in tables {
        let mut stats = Vec::new();
        for s in &LOAD_FACTORS {
            let mut stats_for_this = (0_f32, 0_f64, 0_f32, 0_f64);
            for _ in 0..ITERATIONS_PER_LOAD_FACTOR {
                let temp = get_stats(builder.as_ref(), *s);
                stats_for_this.0 += temp.0;
                stats_for_this.1 += temp.1;
                stats_for_this.2 += temp.2;
                stats_for_this.3 += temp.3;
            }
            stats_for_this.0 /= ITERATIONS_PER_LOAD_FACTOR as f32;
            stats_for_this.1 /= ITERATIONS_PER_LOAD_FACTOR as f64;
            stats_for_this.2 /= ITERATIONS_PER_LOAD_FACTOR as f32;
            stats_for_this.3 /= ITERATIONS_PER_LOAD_FACTOR as f64;
            stats.push(stats_for_this);
        }
        // print stats for this table
        print_subtable(&name, &stats, &LOAD_FACTORS);
        all_stats.push((name, stats));
    }

    // create output file for analysis in csv format
    write_csv(&all_stats, &LOAD_FACTORS);

    // create graph for every type of HashTable
    write_graphs(&all_stats, &LOAD_FACTORS, ELEMENT_COUNT);
}

/// get stats for one type of hash table
///
/// fills the HashTable with `fill` values and then takes measurements
/// for different statistics
fn get_stats(builder: &dyn HashTableBuilder<u32>, fill: f64) -> (f32, f64, f32, f64) {
    let fill = f64::min(fill * ELEMENT_COUNT as f64, ELEMENT_COUNT as f64) as usize;
    get_stats_rec(builder, fill, 0)
}

/// recursively tries to get stats
///
/// fills the HashTable with `fill` values and then takes measurements
/// for different statistics. If it fails at any point it tries again.
/// One reason for failure could be a nearly full OpenAddressingTable
/// with QuadraticProbing. At most, 100 attempts are allowed
fn get_stats_rec(
    builder: &dyn HashTableBuilder<u32>,
    fill: usize,
    attempt: usize,
) -> (f32, f64, f32, f64) {
    // amount of samples to test at random
    let random_samples = 1_usize << 16;

    let mut table = builder.build();
    // resize if needed
    if RESIZE_TO_MAKE_FAIR {
        table.as_mut().resize_to_bytes(ELEMENT_COUNT << 3, fill);
    }

    // fill hash set with `fill` random values
    let mut rng = thread_rng();
    let mut inserted_nums = Vec::with_capacity(fill);
    for _ in 0..fill {
        let num = rng.gen();
        inserted_nums.push(num);
        if !HashTable::insert(table.as_mut(), &num) {
            // try again, up to 100 times
            if attempt > 100 {
                return (std::f32::NAN, std::f64::NAN, std::f32::NAN, std::f64::NAN);
            }
            return get_stats_rec(builder, fill, attempt + 1);
        }
    }
    let mut ns = 0_usize; // number of successful reads
    let mut nf = 0_usize; // number of failed reads
    let mut cs = 0_usize; // collisions on successful reads
    let mut cf = 0_usize; // collisions on failed reads
    let start_time = Instant::now();
    for x in &inserted_nums {
        table.as_mut().has(x);
    }
    // duration of `fill` successful reads
    let duration_s = start_time.elapsed().as_nanos();
    let start_time = Instant::now();
    for _ in 0..random_samples {
        let num = rng.gen();
        table.as_mut().has(&num);
    }
    // we assume random numbers nearly always fail
    // therefore: duration of `random_samples` failed reads
    let duration_f = start_time.elapsed().as_nanos();

    // First try all numbers we already inserted, so we guarantee
    // some successful reads
    for x in &inserted_nums {
        HashTable::reset_collisions(table.as_mut());
        if HashTable::has(table.as_mut(), x) {
            ns += 1;
            cs += HashTable::get_collisions(table.as_ref());
        } else {
            println!("did not find what we would need to find");
            nf += 1;
            cf += HashTable::get_collisions(table.as_ref());
        }
    }
    // Then always try 2^16 more reads with random samples
    for _ in 0..(1_usize << 16) {
        let num = rng.gen();
        HashTable::reset_collisions(table.as_mut());
        if HashTable::has(table.as_mut(), &num) {
            ns += 1;
            cs += HashTable::get_collisions(table.as_ref());
        } else {
            nf += 1;
            cf += HashTable::get_collisions(table.as_ref());
        }
    }
    let nf = nf as f32;
    let cf = cf as f32;
    let ns = ns as f32;
    let cs = cs as f32;
    (
        (cs / ns), // average number of collisions on success
        (duration_s as f64 / fill as f64), // average time on success
        (cf / nf), // average number of collisions on fail
        (duration_f as f64 / random_samples as f64), // average time on fail
    )
}
