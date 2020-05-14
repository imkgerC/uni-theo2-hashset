struct ModHash;
impl Hasher<u32> for ModHash {
    fn hash(val: &u32, max: usize) -> usize {
        *val as usize % max
    }
}

struct MulHash;
const phi: f32 = 0.61803398875;
impl Hasher<u32> for MulHash {
    fn hash(val: &u32, max: usize) -> usize {
        let val = *val as f32;
        (max as f32 * (val * phi - f32::floor(val * phi))) as usize
    }
}

/// Minimal hashing trait
trait Hasher<T> {
    fn hash(val: &T, max: usize) -> usize;
}

trait HashTable<T, H: Hasher<T>> {
    fn has(&self, val: &T) -> bool;
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> u32;
    fn insert(&mut self, val: &T);
}

trait Prober<T> {
    fn probe(i: usize, max: usize, key: &T) -> usize;
}

struct LinearProber;
impl<T> Prober<T> for LinearProber {
    fn probe(i: usize, max: usize, key: &T) -> usize {
        0
    }
}

trait OpenHashTable<T, H: Hasher<T>, P: Prober<T>> {
    fn get(&self, index: usize) -> Option<T>;
    fn get_max(&self) -> usize;
    fn set(&mut self, index: usize, val: &T);
    fn reset_collisions(&mut self);
    fn get_collisions(&self) -> u32;
    fn probe(&self, key: &T) -> Option<usize> {
        let mut index = H::hash(key, self.get_max());
        let mut attempts = 0;
        while attempts < self.get_max() {
            if self.get(index).is_none() {
                return Some(index);
            }
            attempts += 1;
            index = P::probe(attempts, self.get_max(), key);
        }
        None
    }
}

impl<T: PartialEq, H: Hasher<T>, P: Prober<T>> HashTable<T, H> for OpenHashTable<T, H, P> {
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
            index = P::probe(attempts, self.get_max(), val);
        }
        return false;
    }
    fn reset_collisions(&mut self) {
        OpenHashTable::reset_collisions(self)
    }
    fn get_collisions(&self) -> u32 {
        OpenHashTable::get_collisions(self)
    }
    fn insert(&mut self, val: &T) {
        if let Some(index) = self.probe(val) {
            self.set(index, val);
        }
    }
}
