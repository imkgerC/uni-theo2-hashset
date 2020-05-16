extern crate rand;

mod hashset;

use hashset::*;
use rand::{thread_rng, Rng};

fn main() {
    let table: OpenAddressingTable<u32, QuadraticProber, MulHash> = OpenAddressingTable::new();
    let table: Box<dyn HashTable<u32>> = Box::new(table);
    let tables: Vec<Box<dyn HashTable<u32>>> = Vec::new();
    generate_stats(table, "Lala".to_owned());
}

fn generate_stats(mut table: Box<dyn HashTable<u32>>, name: String) {
    let mut rng = thread_rng();
    for _ in 0..512 {
        // more or less 50% occupancy
        let num = rng.gen();
        HashTable::insert(table.as_mut(), &num);
    }
    let mut ns = 0usize;
    let mut nf = 0usize;
    let mut cs = 0usize;
    let mut cf = 0usize;
    for _ in 0..(1usize << 16) {
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
    println!("Ratio for failures: {}", (cf as f32 / nf as f32));
    println!("Ratio for success: {}", (cs as f32 / ns as f32));
}
