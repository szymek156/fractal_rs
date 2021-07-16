//! Attemp to create a custom floating type for fractal

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Sub};

// clone + copy to be able to do: x + x etc.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct SoftFloat {
    // TODO: that could be a base 2, not 10, but keep it simple for now
    positive: bool,
    // TODO: add bias
    exponent: i32,
    // TODO: When switch to base 2 - we could skip first non zero bit
    significand: u64,
}

/// Get nice "float like" representation
impl fmt::Display for SoftFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut with_dec = String::new();

        if self.exponent < 0 {
            // number is < |1|
            // Add zeros
            with_dec.push_str("0.");
            with_dec.push_str(&"0".repeat((self.exponent.abs()) as usize));
            with_dec.push_str(&self.significand.to_string());
        } else {
            // number is >= |1|
            with_dec = self.significand.to_string();

            with_dec.insert((self.exponent) as usize, '.');
        }

        write!(f, "{}{}", if self.positive { "" } else { "-" }, with_dec)
    }
}

impl From<f64> for SoftFloat {
    fn from(a: f64) -> Self {
        let mut exponent = 0;
        let mut positive = true;
        // Normalize exponent, shift significand as long as it is less than base implementation is operating
        // (< 10 in this case).

        // TODO: it's not perfect but enough for now
        let mut normalized = a;
        if normalized < 0.0 {
            positive = false;
            normalized = normalized.abs();
        }

        if normalized != 0.0 {
            while normalized >= 10.0 {
                normalized /= 10.0;
                exponent += 1;
            }

            while normalized < 1.0 {
                normalized *= 10.0;
                exponent -= 1;
            }
        }

        // Panic, if invalid number if given
        println!("normalized {}", normalized);

        let significand: u64 = format!("{}", normalized).replace('.', "").parse().unwrap();
        Self {
            positive,
            exponent,
            significand,
        }
    }
}

impl Add for SoftFloat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        todo!();
    }
}

impl Sub for SoftFloat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!();
    }
}

impl Mul for SoftFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let positive = self.positive == rhs.positive;
        let exponent = self.exponent + rhs.exponent;
        // TODO: figure out what happens in case of overflow
        let significand = self.significand * rhs.significand;

        SoftFloat {
            positive,
            exponent,
            significand,
        }
    }
}

impl PartialOrd for SoftFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.positive == other.positive {
            // The same sign
            let mut a = &self;
            let mut b = &other;

            if !self.positive {
                // Comparing negative numbers, changed order of "self" and "other" in cmp call
                b = &self;
                a = &other;
            }

            // Comparing positive numbers
            if self.exponent == other.exponent {
                // sign and exponend are the same, compare the value
                return Some(a.significand.cmp(&b.significand));
            }

            // Exponents differ - return which one is bigger
            return Some(a.exponent.cmp(&b.exponent));
        } else {
            // Signs differ
            if self.positive {
                // other is negative
                return Some(Ordering::Greater);
            } else {
                return Some(Ordering::Less);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string_exp_negative_works() {
        // 0.0000125
        let sf = SoftFloat {
            positive: true,
            exponent: -4,
            significand: 125,
        };

        assert_eq!("0.0000125", format!("{}", sf));
    }

    #[test]
    fn to_string_exp_positive_works() {
        // 60.89523
        let sf = SoftFloat {
            positive: true,
            exponent: 2,
            significand: 6089523,
        };

        assert_eq!("60.89523", format!("{}", sf));
    }

    #[test]
    fn to_string_negative_number_works() {
        // 60.89523
        let sf = SoftFloat {
            positive: false,
            exponent: 2,
            significand: 6089523,
        };

        assert_eq!("-60.89523", format!("{}", sf));
    }

    #[test]
    fn convert_from_float_works() {
        assert_eq!(
            SoftFloat::from(100.0),
            SoftFloat {
                positive: true,
                exponent: 2,
                significand: 1
            }
        );
        assert_eq!(
            SoftFloat::from(10.0),
            SoftFloat {
                positive: true,
                exponent: 1,
                significand: 1
            }
        );

        assert_eq!(
            SoftFloat::from(1.0),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 1
            }
        );

        assert_eq!(
            SoftFloat::from(0.0),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 0
            }
        );

        assert_eq!(
            SoftFloat::from(0.1),
            SoftFloat {
                positive: true,
                exponent: -1,
                significand: 1
            }
        );

        assert_eq!(
            SoftFloat::from(0.01),
            SoftFloat {
                positive: true,
                exponent: -2,
                significand: 1
            }
        );

        assert_eq!(
            SoftFloat::from(0.001),
            SoftFloat {
                positive: true,
                exponent: -3,
                significand: 1
            }
        );
    }

    #[test]
    #[ignore]
    fn convert_from_float_which_losses_precision_works() {
        let sf = SoftFloat::from(-123456978.000069696969);
        assert_eq!("-123456978.000069696969", format!("{}", sf));
    }

    #[test]
    fn comparsion_works() {
        let a = SoftFloat::from(100.2345);
        let b = SoftFloat::from(100.2346);
        let c = SoftFloat::from(1.2346);
        let d = SoftFloat::from(-99999.99999);
        let e = SoftFloat::from(-99999.99998);
        let f = SoftFloat::from(-0.098);

        // Positive numbers
        assert!(a < b);
        assert!(a == a);
        assert!(b > a);
        assert!(b > c);
        assert!(c < a);

        // Mixed numbers
        assert!(d < c);
        assert!(c > d);

        // Negative numbers
        assert!(f == f);
        assert!(e < f);
        assert!(f > e);
        assert!(d < e);
    }

    #[test]
    fn multiplication_works() {
        let a = SoftFloat::from(25.0);
        let b = SoftFloat::from(0.5);

        assert_eq!(a * b, SoftFloat::from(12.5));
    }
}
