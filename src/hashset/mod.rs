mod chainingtable;
mod coalescedtable;
mod hashing;
mod openaddressing;
mod probing;

pub use chainingtable::*;
pub use coalescedtable::*;
pub use hashing::*;
pub use openaddressing::*;
pub use probing::*;
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

impl<T: PartialEq, H: HashTable<T> + Default> Default for DefaultHashTableBuilder<T, H> {
    fn default() -> Self {
        Self {
            table: PhantomData,
            t: PhantomData,
        }
    }
}
