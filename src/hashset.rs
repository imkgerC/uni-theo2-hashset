use std::marker::PhantomData;

pub struct ModHash;
impl Hasher<u32> for ModHash {
    fn hash(val: &u32, max: usize) -> usize {
        *val as usize % max
    }
}

pub struct MulHash;
const phi: f64 = 0.61803398875;
impl Hasher<u32> for MulHash {
    fn hash(val: &u32, max: usize) -> usize {
        let val = *val as f64;
        (max as f64 * ((val * phi) - f64::floor(val * phi))) as usize
    }
}

/// Minimal hashing trait
pub trait Hasher<T> {
    fn hash(val: &T, max: usize) -> usize;
}

pub trait HashTable<T> {
    fn has(&mut self, val: &T) -> bool;
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> usize;
    fn insert(&mut self, val: &T);
}

pub trait Prober {
    fn probe(i: usize) -> usize;
}

pub struct LinearProber;
impl Prober for LinearProber {
    fn probe(i: usize) -> usize {
        i
    }
}

pub struct QuadraticProber;
impl Prober for QuadraticProber {
    fn probe(i: usize) -> usize {
        i * i
    }
}

pub trait OpenHashTable<T, H: Hasher<T>, P: Prober> {
    fn get(&self, index: usize) -> Option<T>;
    fn get_max(&self) -> usize;
    fn set(&mut self, index: usize, val: &T);
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> usize;
    fn increment_collisions(&mut self);
    
}

pub struct OpenAddressingTable<T: PartialEq + Copy, P: Prober, H: Hasher<T>> {
    collisions: usize,
    entries: [Option<T>; 1 << 10],
    prober: PhantomData<P>,
    hasher: PhantomData<H>,
}

impl<T: PartialEq + Copy, P: Prober, H: Hasher<T>> OpenAddressingTable<T, P, H> {
    pub fn new() -> Self {
        Self {
            collisions: 0,
            entries: [None; 1 << 10],
            prober: PhantomData,
            hasher: PhantomData,
        }
    }
    fn get(&self, index: usize) -> Option<T> {
        self.entries[index]
    }
    fn get_max(&self) -> usize {
        1 << 10
    }
    fn set(&mut self, index: usize, val: &T) {
        self.entries[index] = Some(*val)
    }
    
    fn increment_collisions(&mut self) {
        self.collisions += 1;
    }
}

impl<T: PartialEq+Copy, H: Hasher<T>, P: Prober> HashTable<T> for OpenAddressingTable<T, P, H>
{
    fn has(&mut self, val: &T) -> bool {
        let mut index = H::hash(val, self.get_max());
        let mut attempts = 0;
        while attempts < self.get_max() {
            if let Some(inside) = self.get(index) {
                if inside == *val {
                    return true;
                }
            } else {
                return false;
            }
            attempts += 1;
            self.increment_collisions();
            index = (index + P::probe(attempts)) % self.get_max();
        }
        return false;
    }
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
    fn insert(&mut self, val: &T) {
        if self.has(val) {
            return;
        }
        let mut index = H::hash(val, self.get_max());
        let mut attempts = 0;
        while attempts < self.get_max() {
            if self.get(index).is_none() {
                self.set(index, val);
                return;
            }
            attempts += 1;
            index = (index + P::probe(attempts)) % self.get_max();
        }
        panic!("key cannot be inserted");
    }
}