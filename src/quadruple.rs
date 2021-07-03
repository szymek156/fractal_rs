//! Quadruple (double the double) implementation
//! float-float operators on graphics hardware:
// http://hal.archives-ouvertes.fr/docs/00/06/33/56/PDF/float-float.pdf
//! Extended-Precision Floating-Point Numbers forGPU Computation:
//! http://andrewthall.org/papers/df64_qf128.pdf

use std::ops::{Add, Mul, Sub};

// For float p = 24, double p = 53 ((2 << 27) + 1).
const SPLIT: f64 = ((2 << 27) + 1) as f64;

#[derive(Debug, Default)]
pub struct Quad {
    low: f64,
    high: f64,
}

// Variable naming taken from publication, don't judge me!

impl Quad {
    pub fn new(low: f64, high: f64) -> Self {
        Self { low, high }
    }

    fn add12(a: f64, b: f64) -> Self {
        let s = a + b;

        let v = s - a;

        let r = (a - (s - v)) + (b - v);

        Self::new(s, r)
    }

    fn mul12(a: f64, b: f64) -> Self {
        let a_quad = Self::from(a);

        let b_quad = Self::from(b);

        let ab = a * b;

        let err1 = ab - (a_quad.high * b_quad.high);

        let err2 = err1 - (a_quad.low * b_quad.high);

        let err3 = err2 - (a_quad.high * b_quad.low);

        Self::new(ab, (a_quad.low * b_quad.low) - err3)
    }
}

impl From<f64> for Quad {
    fn from(a: f64) -> Self {
        let c = SPLIT * a;
        let aBig = c - a;

        let high = c - aBig;

        let low = a - high;

        Self { high, low }
    }
}

/// Operator +
impl Add for Quad {
    type Output = Self;

    // TODO: self, of &self?
    fn add(self, rhs: Self) -> Self {
        let r = self.high + rhs.high;

        let mut s;
        // TODO: try to get rid off this if
        if self.high.abs() >= rhs.high.abs() {
            s = self.high - r + rhs.high + rhs.low + self.low;
        } else {
            s = rhs.high - r + self.high + self.low + rhs.low;
        }

        Self::add12(r, s)
    }
}

impl Mul for Quad {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
    
        // TODO: Error in publication, not sure how t3 should be used?
        let abh = Self::mul12(self.high, rhs.high);
        let t3 = (self.high * rhs.low) * (self.low * rhs.high) + abh.low;

        Self::add12(abh.high, t3)
    }
}

// TODO: Partial ordering:
// https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html