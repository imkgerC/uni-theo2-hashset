pub struct ModHash;
impl Hasher<u32> for ModHash {
    fn hash(val: &u32, max: usize) -> usize {
        *val as usize % max
    }
}

pub struct MulHash;
const PHI: f64 = 0.61803398875;
impl Hasher<u32> for MulHash {
    fn hash(val: &u32, max: usize) -> usize {
        let val = *val as f64;
        (max as f64 * ((val * PHI) - f64::floor(val * PHI))) as usize
    }
}

pub struct XorShiftHash;
impl Hasher<u32> for XorShiftHash {
    fn hash(val: &u32, max: usize) -> usize {
        let x = *val;
        let x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3bu32);
        let x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3bu32);
        let x = (x >> 16) ^ x;
        return x as usize % max;
    }
}

/// Minimal hashing trait
pub trait Hasher<T> {
    fn hash(val: &T, max: usize) -> usize;
}