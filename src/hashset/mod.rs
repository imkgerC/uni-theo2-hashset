//! Module containing everything relevant to hashsets
//!
//! This contains hashing functions, probing functions,
//! HashTable implementations and HashTable builders

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

/// Number of buckets for the OpenAddressingTable and load_factor is based on
pub const ELEMENT_COUNT: usize = 1 << 15;

/// Generic HashTable as set datastructure
///
/// The hashtable should only count collisions on calls for finding an element
/// not on insertion. Every inserted element should only be saved once,
/// as only either having or not having the element is checked.
/// The HashTable can not delete any entries or dynamically resize the table
pub trait HashTable<T> {
    /// checks if the element is in the set
    ///
    /// Should return false if the element can not be found and true if it can
    /// Needs to count every collision that occured during the check
    fn has(&mut self, val: &T) -> bool;
    /// resets collisions
    fn reset_collisions(&mut self);
    /// returns the number of collisions
    fn get_collisions(&self) -> usize;
    /// inserts the element in the HashTable
    ///
    /// returns true if the element is already in the HashTable
    /// returns true if the element was successfully inserted
    /// returns false iff the element cannot be inserted into the HashTable
    ///
    /// ## Causes for failure
    /// - the hashset is full
    /// - a bad cycle in probing hindered insertion
    fn insert(&mut self, val: &T) -> bool;
    /// resize the number of buckets to most closely match the number of bytes used
    ///
    /// depending on the type of HashTable it could be hard to implement with
    /// 100% accuracy. Therefore the expected value of the number
    /// of bytes can be used as an approximate value
    fn resize_to_bytes(&mut self, bytes: usize, elements: usize);
}

/// A generic builder for HashTables
///
/// The trait can be used when a generic HashTable type is to be
/// used as a function parameter
pub trait HashTableBuilder<T> {
    /// returns an instance of HashTable
    fn build(&self) -> Box<dyn HashTable<T>>;
}

/// Default implementation of HashTableBuilder
///
/// Just calls the Default trait of the inner HashTable type.
/// Is the most basic implementation of a HashTableBuilder
/// Should be the most used one
pub struct DefaultHashTableBuilder<T: PartialEq, H: HashTable<T> + Default> {
    // PhantomData is used to not actually save any data
    // It is only used to save the type to be instantiated
    table: PhantomData<H>,
    t: PhantomData<T>,
}

impl<T: PartialEq, H: 'static + HashTable<T> + Default> HashTableBuilder<T>
    for DefaultHashTableBuilder<T, H>
{
    /// returns the default instance of HashTable
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
