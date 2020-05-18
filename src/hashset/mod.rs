mod hashing;
mod probing;

pub use hashing::*;
pub use probing::*;
use std::collections::LinkedList;
use std::marker::PhantomData;

pub const ELEMENT_COUNT: usize = 1 << 15;

pub trait HashTable<T> {
    fn has(&mut self, val: &T) -> bool;
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> usize;
    fn insert(&mut self, val: &T) -> bool;
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize);
}

pub trait HashTableBuilder<T> {
    fn build(&self) -> Box<dyn HashTable<T>>;
}

pub struct DefaultHashTableBuilder<T: PartialEq, H: HashTable<T> + Default> {
    table: PhantomData<H>,
    t: PhantomData<T>,
}

impl<T: PartialEq, H: 'static + HashTable<T> + Default> HashTableBuilder<T>
    for DefaultHashTableBuilder<T, H>
{
    fn build(&self) -> Box<dyn HashTable<T>> {
        Box::new(H::default())
    }
}

impl<T: PartialEq, H: HashTable<T> + Default> DefaultHashTableBuilder<T, H> {
    pub fn new() -> Self {
        Self {
            table: PhantomData,
            t: PhantomData,
        }
    }
}

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

// one Option<(T, Option<usize>)> has size 24 => Can only use 10.922 buckets to only use 1 << 18 B
pub struct CoalescedTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<Option<(T, Option<usize>)>>,
    hasher: PhantomData<H>,
    cursor: usize,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for CoalescedTable<T, H> {
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> CoalescedTable<T, H> {
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
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
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

    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let entries = bytes / 24;
        if entries < elements {
            panic!("cannot resize that low");
        }
        *self = Self::with_size(entries);
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

// one Option<u32> has size of 8B => 1 << 15 elements have (1 << 18)B size ~262kB
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
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
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
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        if elements > ELEMENT_COUNT {
            panic!("trying to insert more elements than possible by constraint");
        }
        if bytes >> 3 != ELEMENT_COUNT {
            panic!("trying to resize to invalid size");
        }
    }
}
