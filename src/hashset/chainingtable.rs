use super::{HashTable, Hasher, ELEMENT_COUNT};
use std::marker::PhantomData;
use std::mem::size_of;

/// Simple singly-linked List
///
/// This implementation is not optimized for insertion performance
/// but for size. The implementation is naive and should not be used
/// for anything serious.
pub enum LinkedList<T> {
    Cons(T, Box<LinkedList<T>>),
    Nil,
}

impl<T: PartialEq + Copy> LinkedList<T> {
    /// checks if an element is contained in the list
    ///
    /// searches the LinkedList recursively with the canonical approach
    pub fn contains(&self, searched: &T) -> bool {
        match self {
            LinkedList::Cons(val, other) => *val == *searched || other.contains(searched),
            LinkedList::Nil => false,
        }
    }

    /// inserts the Value as a new node at the end of the list
    ///
    /// This function is a really slow implementation. It was only
    /// implemented this way because of simplicity
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

/// Direct chaining implementation of HashTable
///
/// Every bucket is a pointer to a LinkedList that is used for
/// collision resolution. An infinite amount of elements can be
/// inserted into this table
pub struct DirectChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<Box<LinkedList<T>>>,
    hasher: PhantomData<H>,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for DirectChainingTable<T, H> {
    /// initializes HashTable with ELEMENT_COUNT buckets
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> DirectChainingTable<T, H> {
    /// initializes HashTable with `size` buckets
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

impl<T: PartialEq + Copy, H: Hasher<T>> HashTable<T> for DirectChainingTable<T, H> {
    /// checks table for value
    ///
    /// checks by checking the LinkedList at the correct bucket.
    /// Counts the number of collisions
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
    /// resets number of collisions
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    /// returns number of collisions
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    /// inserts the element into the HashTable
    ///
    /// always returns true as it won't fail
    fn insert(&mut self, val: &T) -> bool {
        let index = H::hash(val, self.entries.len());
        if !self.entries[index].contains(val) {
            self.entries[index].push(*val);
        }
        true
    }

    /// resizes the number of buckets to specified byte value
    ///
    /// size of a direct chaining table is
    /// buckets*(size of a bucket) + entries*(size of a node)
    /// (for T = u32) size of a bucket is the size of a
    /// pointer to a LinkedList = 8
    /// size of a node is the size of a LinkedList node = 16
    /// Fails if bytes - 16*elements < 8
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        let list_size = size_of::<LinkedList<T>>();
        let available_bytes = bytes as isize - (list_size * elements) as isize;
        if available_bytes < size_of::<Box<LinkedList<T>>>() as isize {
            panic!("not enough bytes available for the buckets");
        }
        *self = Self::with_size(available_bytes as usize / size_of::<Box<LinkedList<T>>>());
    }
}

/// Separate chaining implementation of HashTable
///
/// Every bucket has stores one value and a pointer to a LinkedList
/// that is used for collision resolution. An infinite amount of
/// elements can be inserted into this table.
pub struct SeparateChainingTable<T: PartialEq + Copy, H: Hasher<T>> {
    collisions: usize,
    entries: Vec<(Option<T>, Box<LinkedList<T>>)>,
    hasher: PhantomData<H>,
}
impl<T: PartialEq + Copy, H: Hasher<T>> Default for SeparateChainingTable<T, H> {
    /// initializes HashTable with ELEMENT_COUNT buckets
    fn default() -> Self {
        Self::with_size(ELEMENT_COUNT)
    }
}

impl<T: PartialEq + Copy, H: Hasher<T>> SeparateChainingTable<T, H> {
    /// initializes HashTable with `size` buckets
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
    /// checks table for value
    ///
    /// checks by checking first checking the value stored at the
    /// correct bucket, then checking the associated LinkedList.
    /// Counts the number of collisions
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
    /// resets number of collisions
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    /// returns number of collisions
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    /// inserts the element into the HashTable
    ///
    /// always returns true as it won't fail
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
    /// resizes the number of buckets to specified byte value
    ///
    /// Warning: Is correct for T=u32 only.
    /// (for T=u32) The size of a separate chaining table is
    /// elements*16 + empty_buckets*16
    /// The number of empty buckets is approximated with m((m-1)/m)^n
    /// As this equation grows monotonically in m, the value can be found
    /// by using a variant of binary search.
    /// Will fail if no suitable value m is found.
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize) {
        // to make calculations simpler we do not calculate in terms of
        // bytes but in terms of 16B always
        let available_elements = (bytes / 16) as f64;

        let elements = elements as f64;
        // We cannot allocate more buckets than with zero collisions
        let mut buckets = available_elements as f64;
        let mut step = available_elements as f64 / 2_f64;
        // do to an accuracy of 1
        while step > 1_f64 {
            let used_elements = buckets * ((buckets - 1_f64) / buckets).powf(elements) + elements;
            if used_elements > available_elements {
                // found a maximum, go lower
                buckets -= step;
            } else if used_elements < available_elements {
                // found a minimum, can lower step size
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
