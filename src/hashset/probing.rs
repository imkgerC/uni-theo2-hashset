pub trait Prober {
    fn probe(i: usize) -> usize;
}

pub struct TriangularProber;
impl Prober for TriangularProber {
    fn probe(i: usize) -> usize {
        (i * (i + 1)) >> 1
    }
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