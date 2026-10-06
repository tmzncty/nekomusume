//! Deterministic PRNG: xorshift64* seeded from a u64.
//!
//! Zero dependencies (no `rand`), fully reproducible under `--seed`.
//! Quality is more than sufficient for impairment drawing; it is NOT
//! cryptographic and does not try to be.

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seeds must be nonzero; 0 is remapped to the splitmix constant so the
    /// sequence never degenerates.
    pub fn new(seed: u64) -> Self {
        Rng { state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed } }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        debug_assert_ne!(x, 0);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform f64 in [0, 1). 53 explicit bits from the top of the word.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64))
    }

    /// Uniform integer in [0, n). Panics on n == 0.
    #[inline]
    pub fn next_below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "next_below(0)");
        // Multiply-shift (Lemire): essentially no modulo bias for our sizes.
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_diverges() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        let mut diff = 0;
        for _ in 0..100 {
            if a.next_u64() != b.next_u64() {
                diff += 1;
            }
        }
        assert!(diff > 50, "sequences unexpectedly correlated: {diff}");
    }

    #[test]
    fn zero_seed_does_not_stick() {
        let mut r = Rng::new(0);
        assert_ne!(r.next_u64(), 0);
        assert_ne!(r.next_u64(), 0);
    }

    #[test]
    fn f64_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..100_000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn uniform_mean_converges() {
        // p=10% Bernoulli over 100k draws must land in [9.5%, 10.5%].
        let mut r = Rng::new(20261007);
        let n = 100_000;
        let hits = (0..n).filter(|_| r.next_f64() < 0.10).count();
        let p = hits as f64 / n as f64;
        assert!((0.095..=0.105).contains(&p), "p={p}");
    }
}
