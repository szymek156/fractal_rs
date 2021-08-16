//! Quadruple (double the double) implementation
//! Double single functions based on DSFUN90 package:
//! http://crd.lbl.gov/~dhbailey/mpdist/index.html

//! Other references
//! float-float operators on graphics hardware:
// http://hal.archives-ouvertes.fr/docs/00/06/33/56/PDF/float-float.pdf
//! Extended-Precision Floating-Point Numbers forGPU Computation:
//! http://andrewthall.org/papers/df64_qf128.pdf

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

// For float p = 24, double p = 53 ((2 << 27) + 1).
const SPLIT: f64 = ((2 << 27) + 1) as f64;

// clone + copy to be able to do: x + x etc.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Quad {
    pub lo: f64,
    pub hi: f64,
}

// Variable naming taken from publication, don't judge me!

impl Quad {
    pub fn new(lo: f64, hi: f64) -> Self {
        Self { lo, hi }
    }
}

impl From<f64> for Quad {
    fn from(a: f64) -> Self {
        Self { hi: a, lo: 0.0 }
    }
}

/// Operator +
impl Add for Quad {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        // Compute dsa + dsb using Knuth's trick.
        let t1 = self.hi + rhs.hi;
        let mut e = t1 - self.hi;
        let t2 = ((rhs.hi - e) + (self.hi - (t1 - e))) + self.lo + rhs.lo;

        // The result is t1 + t2, after normalization.
        e = t1 + t2;
        let c0 = e;
        let c1 = t2 - (e - t1);

        Quad { lo: c1, hi: c0 }
    }
}

impl AddAssign for Quad {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Quad {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Compute dsa - dsb using Knuth's trick.
        let t1 = self.hi - rhs.hi;
        let mut e = t1 - self.hi;
        let t2 = ((-rhs.hi - e) + (self.hi - (t1 - e))) + self.lo - rhs.lo;

        // The result is t1 + t2, after normalization.
        e = t1 + t2;
        let c0 = e;
        let c1 = t2 - (e - t1);

        Quad { lo: c1, hi: c0 }
    }
}

impl SubAssign for Quad {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Quad {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // This splits dsa(1) and dsb(1) into high-order and low-order words.
        let cona = self.hi * SPLIT;
        let conb = rhs.hi * 8193.0;
        let sa1 = cona - (cona - self.hi);
        let sb1 = conb - (conb - rhs.hi);
        let sa2 = self.hi - sa1;
        let sb2 = rhs.hi - sb1;

        // Multilply a0 * b0 using Dekker's method.
        let c11 = self.hi * rhs.hi;
        let c21 = (((sa1 * sb1 - c11) + sa1 * sb2) + sa2 * sb1) + sa2 * sb2;

        // Compute a0 * b1 + a1 * b0 (only high-order word is needed).
        let c2 = self.hi * rhs.lo + self.lo * rhs.hi;

        // Compute (c11, c21) + c2 using Knuth's trick, also adding low-order product.
        let t1 = c11 + c2;
        let mut e = t1 - c11;
        let t2 = ((c2 - e) + (c11 - (t1 - e))) + c21 + self.lo * rhs.lo;

        // The result is t1 + t2, after normalization.
        e = t1 + t2;
        let c0 = e;
        let c1 = t2 - (e - t1);

        return Quad { lo: c1, hi: c0 };
    }
}

impl MulAssign for Quad {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
// TODO: Partial ordering:
// https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html

impl PartialOrd for Quad {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // return (hi < q.hi) || (hi == q.hi && lo < q.lo);

        if self.hi < other.hi || (self.hi == other.hi && self.lo < other.lo) {
            return Some(Ordering::Less);
        } else {
            return Some(Ordering::Greater);
        }

        // For fractal we care only for < operator, others can be implemented later
        todo!();
    }
}
