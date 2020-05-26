/// Simple modulo hasher
///
/// This is the most simple hashing function one could fathom.
/// It does not calculate modulo a big prime but only modulo the maximum
pub struct ModHash;
impl Hasher<u32> for ModHash {
    fn hash(val: &u32, max: usize) -> usize {
        *val as usize % max
    }
}

/// Theoretically nice multiplicative hasher
///
/// This works by multiplying with the golden ratio, which is the number hardest to
/// approximate with fractions. This makes for a pseudo-random decimal fraction part
/// which is used as the base for the hash. This is then multiplied with the maximum
pub struct MulHash;
const PHI: f64 = 0.618_033_988_75;
impl Hasher<u32> for MulHash {
    fn hash(val: &u32, max: usize) -> usize {
        let val = *val as f64;
        (max as f64 * ((val * PHI) - f64::floor(val * PHI))) as usize
    }
}

/// XOR shift hasher
///
/// This works by using the value as the seed for a pseudo-random XORShiftRng
pub struct XorShiftHash;
impl Hasher<u32> for XorShiftHash {
    fn hash(val: &u32, max: usize) -> usize {
        let x = *val;
        let x = ((x >> 16) ^ x).wrapping_mul(0x45d_9f3b_u32);
        let x = ((x >> 16) ^ x).wrapping_mul(0x45d_9f3b_u32);
        let x = (x >> 16) ^ x;
        x as usize % max
    }
}

/// Minimal trait for hashing functions
///
/// Every hasher is a non-instantiable struct with a static method for hashing
/// The hashing trait is implemented for a specific type
pub trait Hasher<T> {
    /// Hashing function
    ///
    /// - val: A reference to the value to hash
    /// - max: The length of the hashset
    /// returns: An integer value in the interval [0, max)
    fn hash(val: &T, max: usize) -> usize;
}
