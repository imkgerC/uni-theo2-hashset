/// Trait for Prober for OpenAdressingTables
///
/// The Prober provides the offset to add at the ith attempt
pub trait Prober {
    /// Provides the offset at the ith attempt
    ///
    /// The implementor should provide an offset. There are no limitations on the range of the offset
    fn probe(i: usize) -> usize;
}

/// Triangular Probing
///
/// The triangular prober always has the sum from 0 to i as the offset
/// It uses the gaussian formula for summation. This approach is all used in
/// real world high-performance implementations
pub struct TriangularProber;
impl Prober for TriangularProber {
    fn probe(i: usize) -> usize {
        (i * (i + 1)) >> 1 // sum (0, .., i) = (i(i+1))/2
    }
}

/// Simplest prober
///
/// The offset is i. It tries all buckets in linear fashion
pub struct LinearProber;
impl Prober for LinearProber {
    fn probe(i: usize) -> usize {
        i
    }
}

/// Quadratic probing is not used in pracitcal applications anymore
///
/// The offset is simple i*i. This makes for an easy implementation with less clustering than linear probing.
/// An issue with this method is that not every value forms a cycle over every bucket, so the table may become full before
/// every bucket is used.
pub struct QuadraticProber;
impl Prober for QuadraticProber {
    fn probe(i: usize) -> usize {
        i * i
    }
}
