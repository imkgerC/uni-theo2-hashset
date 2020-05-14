pub struct ModHash;
impl Hasher<u32> for ModHash {
    fn hash(val: &u32, max: usize) -> usize {
        *val as usize % max
    }
}

pub struct MulHash;
const phi: f32 = 0.61803398875;
impl Hasher<u32> for MulHash {
    fn hash(val: &u32, max: usize) -> usize {
        let val = *val as f32;
        (max as f32 * (val * phi - f32::floor(val * phi))) as usize
    }
}

/// Minimal hashing trait
pub trait Hasher<T> {
    fn hash(val: &T, max: usize) -> usize;
}

pub trait HashTable<T, H: Hasher<T>> {
    fn has(&self, val: &T) -> bool;
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
    fn probe(&self, key: &T) -> Option<usize> {
        let mut index = H::hash(key, self.get_max());
        let mut attempts = 0;
        while attempts < self.get_max() {
            if self.get(index).is_none() {
                return Some(index);
            }
            attempts += 1;
            index = (index + P::probe(attempts)) % self.get_max();
        }
        None
    }
}

impl<T: PartialEq, H: Hasher<T>, P: Prober> HashTable<T, H> for dyn OpenHashTable<T, H, P> {
    fn has(&self, val: &T) -> bool {
        let mut index = H::hash(val, self.get_max());
        let mut attempts = 0;
        while attempts < self.get_max() {
            if let Some(inside) = self.get(index) {
                if inside == *val {
                    return true;
                }
            }
            attempts += 1;
            index = (index + P::probe(attempts)) % self.get_max();
        }
        return false;
    }
    fn reset_collisions(&mut self) {
        OpenHashTable::reset_collisions(self)
    }
    fn get_collisions(&self) -> usize {
        OpenHashTable::get_collisions(self)
    }
    fn insert(&mut self, val: &T) {
        if self.has(val) {
            return;
        }
        if let Some(index) = self.probe(val) {
            self.set(index, val);
        } else {
            panic!("key cannot be inserted");
        }
    }
}

pub struct OpenTable1024<T: PartialEq + Copy + Default> {
    collisions: usize,
    entries: [Option<T>; 1 << 10],
}

impl<T: PartialEq + Copy + Default> OpenTable1024<T> {
    pub fn new() -> Self {
        Self {
            collisions: 0,
            entries: [Some(T::default()); 1 << 10],
        }
    }
}

impl<T: PartialEq + Copy + Default, P: Prober, H: Hasher<T>> OpenHashTable<T, H, P>
    for OpenTable1024<T>
{
    fn get(&self, index: usize) -> Option<T> {
        self.entries[index]
    }
    fn get_max(&self) -> usize {
        1 << 10
    }
    fn set(&mut self, index: usize, val: &T) {
        self.entries[index] = Some(*val)
    }
    fn reset_collisions(&mut self) {
        self.collisions = 0;
    }
    fn get_collisions(&self) -> usize {
        self.collisions
    }
}
