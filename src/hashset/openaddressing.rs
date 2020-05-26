use super::{HashTable, Hasher, Prober, ELEMENT_COUNT};
use std::marker::PhantomData;

/// Simple and fast HashTable with OpenAddressing
///
/// OpenAddressing is used for collision resolution. The number of
/// buckets is always equal to ELEMENT_COUNT. If quadratic probing is
/// used, insertion could fail even though not every bucket is used.
pub struct OpenAddressingTable<T: PartialEq + Copy, P: Prober, H: Hasher<T>> {
    collisions: usize,
    entries: [Option<T>; ELEMENT_COUNT],
    prober: PhantomData<P>,
    hasher: PhantomData<H>,
}

impl<T: PartialEq + Copy, P: Prober, H: Hasher<T>> Default for OpenAddressingTable<T, P, H> {
    fn default() -> Self {
        Self {
            collisions: 0,
            entries: [None; ELEMENT_COUNT],
            prober: PhantomData,
            hasher: PhantomData,
        }
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>, P: Prober> HashTable<T> for OpenAddressingTable<T, P, H> {
    /// probes table for value
    ///
    /// returns true iff value was inserted into HashTable
    /// It will at maximum check a number of buckets equal to the 
    /// total number of buckets. It does not use cycle detection.
    /// While probing the hash+offset is wrapped around the end of the table.
    /// Every accessed non-empty bucket that did not contain the value
    /// searched for is counted as a collision
    fn has(&mut self, val: &T) -> bool {
        let mut index = H::hash(val, self.entries.len());
        let mut attempts = 0;
        while attempts < self.entries.len() {
            if let Some(inside) = self.entries[index] {
                if inside == *val {
                    return true;
                }
            } else {
                return false;
            }
            attempts += 1;
            self.collisions += 1;
            index = (index + P::probe(attempts)) % self.entries.len();
        }
        false
    }
    /// resets number of collisions
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    /// returns number of collisions
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    /// inserts the element in the HashTable if possible
    /// 
    /// returns true iff the value was inserted successfully
    /// It will at maximum check a number of buckets equal to the 
    /// total number of buckets. It does not use cycle detection.
    /// Insertion is not optimized for performance.
    /// Insertion does not count collisions.
    fn insert(&mut self, val: &T) -> bool {
        if self.has(val) {
            return true;
        }
        let mut index = H::hash(val, self.entries.len());
        let mut attempts = 0;
        while attempts < self.entries.len() {
            if self.entries[index].is_none() {
                self.entries[index] = Some(*val);
                return true;
            }
            attempts += 1;
            index = (index + P::probe(attempts)) % self.entries.len();
        }
        false
    }
    /// fails if bytes unequal to 8*ELEMENT_COUNT (for T = u32)
    ///
    /// As the OpenAddressingTable is backed by an array with static size,
    /// it can't be dynamically allocated or resized. Thusly this
    /// method will fail if not called with bytes = 
    /// ELEMENT_COUNT*size_of(Option<T>) and elements <= ELEMENT_COUNT
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        if elements > ELEMENT_COUNT {
            panic!("trying to insert more elements than possible by constraint");
        }
        if bytes * std::mem::size_of::<Option<T>>() != ELEMENT_COUNT {
            panic!("trying to resize to invalid size");
        }
    }
}
