use super::{HashTable, Hasher, ELEMENT_COUNT};
use std::marker::PhantomData;

enum LinkedList<T> {
    Cons(T, Box<LinkedList<T>>),
    Nil,
}

impl<T: PartialEq + Copy> LinkedList<T> {
    pub fn contains(&self, searched: &T) -> bool {
        match self {
            LinkedList::Cons(val, other) => *val == *searched || other.contains(searched),
            LinkedList::Nil => false,
        }
    }

    pub fn push(&mut self, val: T) {
        let push_to_self = match self {
            LinkedList::Cons(_, other) => {
                other.push(val);
                false
            }
            LinkedList::Nil => true,
        };
        if push_to_self {
            *self = LinkedList::Cons(val, Box::new(LinkedList::Nil));
        }
    }
}

pub struct DirectChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<Box<LinkedList<T>>>,
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
            entries.push(Box::new(LinkedList::Nil));
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
    entries: Vec<(Option<T>, Box<LinkedList<T>>)>,
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
            entries.push((None, Box::new(LinkedList::Nil)));
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
        let mut right_now = self.entries[index].1.as_ref();
        while let LinkedList::Cons(x, new) = right_now {
            if *x == *val {
                return true;
            }
            self.collisions += 1;
            right_now = new.as_ref();
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
        self.entries[index].1.push(*val);
        true
    }
    // the type is kind of complicated for this hash table so some assumptions will be made
    // a bucket has size 16, any element that collides with another one takes up 16
    // if we assume that 0.25 of all elements have collided with another, our formula becomes
    // 16*bytes + 0.25*16*elements
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let available_elements = (bytes / 16) as f64;
        let elements = elements as f64;
        let mut buckets = available_elements as f64;
        let mut step = available_elements as f64 / 2_f64;
        while step > 1_f64 {
            let used_elements = buckets * ((buckets - 1_f64) / buckets).powf(elements) + elements;
            if used_elements > available_elements {
                buckets -= step;
            } else if used_elements < available_elements {
                step = step / 2_f64;
                buckets += step;
            }
        }
        if buckets < 1_f64 {
            panic!("invalid configuration for direct chaining table");
        }
        *self = Self::with_size(buckets as usize);
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> HashTable<T> for DirectChainingTable<T, H> {
    fn has(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        let mut right_now = self.entries[index].as_ref();
        while let LinkedList::Cons(x, next) = right_now {
            if *x == *val {
                return true;
            }
            self.collisions += 1;
            right_now = next.as_ref();
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
            self.entries[index].push(*val);
        }
        true
    }

    // size of direct chaining table is buckets*(size of bucket) + entries*(size of node)
    // size of bucket is the size of a pointer to a linkedlist = 8
    // size of node is the size of a linkedlist node = 16
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let available_bytes = bytes as isize - (16 * elements) as isize;
        if available_bytes < 1 {
            panic!("not enough bytes available for the buckets");
        }
        *self = Self::with_size(available_bytes as usize / 8);
    }
}
