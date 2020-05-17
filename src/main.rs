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

const LOAD_FACTORS: [f64; 5] = [0.2, 0.5, 0.7, 0.9, 0.95];

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
    let mut out = format!("{:20}", "Name");
    for (i, lambda) in LOAD_FACTORS.into_iter().enumerate() {
        let lambda = format!("{:.0}%", lambda * 100f64);
        out.push_str(&format!("{:^5}", lambda));
        if i != LOAD_FACTORS.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    println!("{:-<width$}", "", width = 25 + 5 * LOAD_FACTORS.len());
}

fn print_subtable(name: &str, stats: &[(f32, f64, f32, f64)]) {
    println!("{:20}", name);
    let mut out = format!("{:20}", "+ collisions");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].0));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "+ time[ns]");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].1));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "- collisions");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].2));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
    let mut out = format!("{:20}", "- time[ns]");
    for i in 0..stats.len() {
        out.push_str(&format!("{:^5.2}", stats[i].3));
        if i != stats.len() - 1 {
            out.push_str("|");
        }
    }
    println!("{}", out);
}

fn generate_stats(tables: Vec<(Box<dyn HashTableBuilder<u32>>, String)>) {
    let mut all_stats = Vec::new();
    for (builder, name) in tables {
        let mut stats = [(0f32, 0f64, 0f32, 0f64); LOAD_FACTORS.len()];
        for (i, s) in LOAD_FACTORS.into_iter().enumerate() {
            stats[i] = get_stats(&builder, *s);
        }
        print_subtable(&name, &stats);
        all_stats.push((name, stats));
    }

    // create output file for analysis in csv format
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("hashset_data.csv")
        .expect("Could not open file to write output analysis to");
    let mut header = String::new();
    header.push_str("\"Name\",");
    for lambda in LOAD_FACTORS.into_iter() {
        let percentage = format!("{:.0}%", lambda*100f64);
        header.push_str(&format!("\"Success Collisions({0})\",\"Success Time({0})[ns]\",\"Failures Collisions({0})\",\"Failures Time({0})[ns]\",", percentage));
    }
    header.push_str("\r\n");
    file.write(header.as_bytes())
        .expect("Could not write to file");
    for (name, stats) in all_stats {
        let mut f = format!("\"{}\"", name);
        for stat in stats.iter() {
            f.push_str(&format!(",{},{},{},{}", stat.0, stat.1, stat.2, stat.3));
        }
        f.push_str("\n");
        file.write(f.as_bytes()).expect("Could not write to file");
    }
}

fn get_stats(builder: &Box<dyn HashTableBuilder<u32>>, fill: f64) -> (f32, f64, f32, f64) {
    let fill = f64::min(fill * ELEMENT_COUNT as f64, ELEMENT_COUNT as f64) as usize;
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
        (duration_s as f64 / fill as f64),
        (cf / nf),
        (duration_f as f64 / random_samples as f64),
    )
}
