use super::{HashTable, Hasher, ELEMENT_COUNT};
use std::collections::LinkedList;
use std::marker::PhantomData;

pub struct DirectChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<LinkedList<T>>,
    hasher: PhantomData<H>,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for DirectChainingTable<T, H> {
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> DirectChainingTable<T, H> {
    fn with_size(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(LinkedList::new());
        }
        Self {
            collisions: 0,
            entries,
            hasher: PhantomData,
        }
    }
}

pub struct SeparateChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<(Option<T>, LinkedList<T>)>,
    hasher: PhantomData<H>,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for SeparateChainingTable<T, H> {
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> SeparateChainingTable<T, H> {
    fn with_size(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push((None, LinkedList::new()));
        }
        Self {
            collisions: 0,
            entries,
            hasher: PhantomData,
        }
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> HashTable<T> for SeparateChainingTable<T, H> {
    fn has(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        if let Some(x) = self.entries[index].0 {
            if x == *val {
                return true;
            }
            self.collisions += 1;
        }
        for x in &self.entries[index].1 {
            if *x == *val {
                return true;
            }
            self.collisions += 1;
        }
        false
    }
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    fn insert(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        if self.entries[index].0.is_none() {
            self.entries[index].0 = Some(*val);
            return true;
        }
        if let Some(x) = self.entries[index].0 {
            if x == *val {
                return true;
            }
        }
        if self.entries[index].1.contains(val) {
            return true;
        }
        self.entries[index].1.push_front(*val);
        true
    }
    // the type is kind of complicated for this hash table so some assumptions will be made
    // a bucket has size 40, any element that collides with another one takes up 24
    // if we assume that 0.25 of all elements have collided with another, our formula becomes
    // 40*bytes + 0.25*24*elements
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let available_bytes = bytes as isize - (elements as isize * 6);
        if available_bytes < 1 {
            panic!("invalid configuration for direct chaining table");
        }
        *self = Self::with_size(available_bytes as usize / 40);
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> HashTable<T> for DirectChainingTable<T, H> {
    fn has(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        for x in &self.entries[index] {
            if *x == *val {
                return true;
            }
            self.collisions += 1;
        }
        false
    }
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    fn insert(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        if !self.entries[index].contains(val) {
            self.entries[index].push_front(*val);
        }
        true
    }

    // size of direct chaining table is buckets*(size of bucket) + entries*(size of node)
    // size of bucket is the size of an empty linkedlist = 24
    // size of node is the size of a linkedlist node = 24
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let available_bytes = bytes as isize - (24 * elements) as isize;
        if available_bytes < 1 {
            panic!("not enough bytes available for the buckets");
        }
        *self = Self::with_size(available_bytes as usize / 24);
    }
}
