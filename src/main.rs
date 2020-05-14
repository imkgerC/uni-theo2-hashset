mod hashset;

use hashset::*;
fn main() {
    let mut table: OpenTable1024<u32> = OpenTable1024::new();
    let mut table: Box<dyn OpenHashTable<u32, MulHash, QuadraticProber>> = Box::new(table);
    HashTable::insert(table.as_mut(), &3);
    println!("{}", HashTable::has(table.as_ref(), &3));
}
