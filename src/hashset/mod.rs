mod hashing;
mod probing;

pub use hashing::*;
pub use probing::*;
use std::collections::LinkedList;
use std::marker::PhantomData;

pub trait HashTable<T> {
    fn has(&mut self, val: &T) -> bool;
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> usize;
    fn insert(&mut self, val: &T) -> bool;
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

pub const ELEMENT_COUNT: usize = 1 << 15;

pub struct DirectChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<LinkedList<T>>,
    hasher: PhantomData<H>,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for DirectChainingTable<T, H> {
    fn default() -> Self {
        let mut entries = Vec::with_capacity(ELEMENT_COUNT);
        for _ in 0..ELEMENT_COUNT {
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
        let mut entries = Vec::with_capacity(ELEMENT_COUNT);
        for _ in 0..ELEMENT_COUNT {
            entries.push((None, LinkedList::new()));
        }
        Self {
            collisions: 0,
            entries,
            hasher: PhantomData,
        }
    }
}

pub struct CoalescedTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    pub entries: [Option<(T, Option<usize>)>; ELEMENT_COUNT],
    hasher: PhantomData<H>,
    cursor: usize,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for CoalescedTable<T, H> {
    fn default() -> Self {
        Self {
            collisions: 0,
            entries: [None; ELEMENT_COUNT],
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
}

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
}
