use super::{HashTable, Hasher, ELEMENT_COUNT};
use std::marker::PhantomData;

/// HashTable with coalesced buckets for collision resolution
///
/// Every buckets saves an Element and an optional pointer to
/// the next bucket used for collision resolution.
pub struct CoalescedTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<Option<(T, Option<usize>)>>,
    hasher: PhantomData<H>,
    cursor: usize,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for CoalescedTable<T, H> {
    /// initializes HashTable with ELEMENT_COUNT buckets
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> CoalescedTable<T, H> {
    /// initializes HashTable with `size` buckets
    fn with_size(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(None);
        }
        Self {
            collisions: 0,
            entries,
            hasher: PhantomData,
            cursor: 0,
        }
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> HashTable<T> for CoalescedTable<T, H> {
    /// checks table for value
    ///
    /// checks the table through the efficient algorithm used in
    /// separate chaining.
    fn has(&mut self, val: &T) -> bool {
        let mut index = H::hash(val, self.entries.len());
        if self.entries[index].is_none() {
            self.entries[index] = Some((*val, None));
            return true;
        }
        self.collisions += 1;
        loop {
            if let Some((x, next)) = self.entries[index] {
                if x == *val {
                    return true;
                }
                self.collisions += 1;
                if let Some(i) = next {
                    index = i;
                } else {
                    break;
                }
            } else {
                panic!("data inconsistency");
            }
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

    /// inserts an element into the table
    ///
    /// returns true iff the value was inserted successfully
    /// Only fails iff the table is full and the value was
    /// not inserted already
    /// Insertion is not optimized for performance.
    /// Insertion does not count collisions.
    fn insert(&mut self, val: &T) -> bool {
        let mut index = H::hash(val, self.entries.len());
        if self.entries[index].is_none() {
            self.entries[index] = Some((*val, None));
            return true;
        }
        loop {
            if let Some((x, next)) = self.entries[index] {
                if x == *val {
                    return true;
                }
                if let Some(i) = next {
                    index = i;
                } else {
                    break;
                }
            } else {
                panic!("data inconsistency");
            }
        }
        while self.cursor < self.entries.len() {
            if self.entries[self.cursor].is_none() {
                self.entries[self.cursor] = Some((*val, None));
                let old = self.entries[index].expect("data inconsistency").0;
                self.entries[index] = Some((old, Some(self.cursor)));
                return true;
            }
            self.cursor += 1;
        }
        true
    }
    /// resizes the number of buckets to specified byte value
    ///
    /// (for T = u32) Every bucket has a size of 24B. If elements*24 > bytes
    /// this method will fail. Otherwise it resizes the hashtable to 
    /// bytes / 24 buckets.
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let entries = bytes / std::mem::size_of::<Option<(T, Option<usize>)>>();
        if entries < elements {
            panic!("cannot resize that low");
        }
        *self = Self::with_size(entries);
    }
}
