extern crate rand;

mod hashset;
use hashset::*;
use rand::{thread_rng, Rng};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

fn get_builder<T: PartialEq + 'static, H: 'static + HashTable<T> + Default>(
) -> Box<dyn HashTableBuilder<T>> {
    Box::new(DefaultHashTableBuilder::<T, H>::new())
}

fn main() {
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
    print_header();
    generate_stats(tables);
}

fn print_header() {
    println!(
        "{:20}{:^5}|{:^5}|{:^5}|{:^5}",
        "Name", "50%", "90%", "95%", "100%"
    );
    println!("{:-<45}", "");
}

fn generate_stats(tables: Vec<(Box<dyn HashTableBuilder<u32>>, String)>) {
    let mut all_stats = Vec::new();
    for (builder, name) in tables {
        let mut stats = [(0f32, 0f64, 0f32, 0f64); 4];
        for (i, s) in [512, 921, 973, 1024].into_iter().enumerate() {
            stats[i] = get_stats(&builder, *s);
        }
        println!("{:20}", name);
        println!(
            "{:20}{:^5.2}|{:^5.2}|{:^5.2}|{:^5.2}",
            "+ collisions", stats[0].0, stats[1].0, stats[2].0, stats[3].0
        );
        println!(
            "{:20}{:^5.2}|{:^5.2}|{:^5.2}|{:^5.2}",
            "+ time[ns]", stats[0].1, stats[1].1, stats[2].1, stats[3].1
        );
        println!(
            "{:20}{:^5.2}|{:^5.2}|{:^5.2}|{:^5.2}",
            "- collisions", stats[0].2, stats[1].2, stats[2].2, stats[3].2
        );
        println!(
            "{:20}{:^5.2}|{:^5.2}|{:^5.2}|{:^5.2}",
            "- time[ns]", stats[0].3, stats[1].3, stats[2].3, stats[3].3
        );
        all_stats.push((name, stats));
    }

    // create output file for analysis in csv format
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("hashset_data.csv")
        .expect("Could not open file to write output analysis to");
    let percentages = ["50%", "90%", "95%", "100%"];
    let mut header = String::new();
    header.push_str("\"Name\",");
    for percentage in &percentages {
        header.push_str(&format!("\"Success Collisions({0})\",\"Success Time({0})[ns]\",\"Failures Collisions({0})\",\"Failures Time({0})[ns]\",", percentage));
    }
    header.push_str("\r\n");
    file.write(header.as_bytes())
        .expect("Could not write to file");
    for (name, stats) in all_stats {
        let f = format!(
            "\"{}\",{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            name,
            stats[0].0,
            stats[0].1,
            stats[0].2,
            stats[0].3,
            stats[1].0,
            stats[1].1,
            stats[1].2,
            stats[1].3,
            stats[2].0,
            stats[2].1,
            stats[2].2,
            stats[2].3,
            stats[3].0,
            stats[3].1,
            stats[3].2,
            stats[3].3
        );
        file.write(f.as_bytes()).expect("Could not write to file");
    }
}

fn get_stats(builder: &Box<dyn HashTableBuilder<u32>>, fill: usize) -> (f32, f64, f32, f64) {
    get_stats_rec(builder, fill, 0)
}

fn get_stats_rec(
    builder: &Box<dyn HashTableBuilder<u32>>,
    fill: usize,
    attempt: usize,
) -> (f32, f64, f32, f64) {
    let mut table = builder.build();
    let mut rng = thread_rng();
    let mut inserted_nums = Vec::with_capacity(fill);
    for _ in 0..fill {
        // more or less 50% occupancy
        let num = rng.gen();
        inserted_nums.push(num);
        if !HashTable::insert(table.as_mut(), &num) {
            if attempt > 100 {
                return (std::f32::NAN, std::f64::NAN, std::f32::NAN, std::f64::NAN);
            }
            return get_stats_rec(builder, fill, attempt + 1);
        }
    }
    let mut ns = 0usize;
    let mut nf = 0usize;
    let mut cs = 0usize;
    let mut cf = 0usize;
    let random_samples = 1usize << 16;
    let start_time = Instant::now();
    for x in &inserted_nums {
        table.as_mut().has(x);
    }
    let duration_s = start_time.elapsed().as_nanos();
    let start_time = Instant::now();
    for _ in 0..random_samples {
        let num = rng.gen();
        table.as_mut().has(&num);
    }
    let duration_f = start_time.elapsed().as_nanos();
    for x in inserted_nums {
        HashTable::reset_collisions(table.as_mut());
        if HashTable::has(table.as_mut(), &x) {
            ns += 1;
            cs += HashTable::get_collisions(table.as_ref());
        } else {
            println!("did not find what we would need to find");
            nf += 1;
            cf += HashTable::get_collisions(table.as_ref());
        }
    }
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
    let nf = nf as f32;
    let cf = cf as f32;
    let ns = ns as f32;
    let cs = cs as f32;
    (
        (cs / ns),
        (duration_s as f64 / ns as f64),
        (cf / nf),
        (duration_f as f64 / random_samples as f64),
    )
}
