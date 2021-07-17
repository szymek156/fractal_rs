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
    // Uses normalization but requires significand to be float - that's stupid!
    // fn from(a: f64) -> Self {
    //     let mut exponent = 0;
    //     let mut positive = true;
    //     // Normalize exponent, shift significand as long as it is less than base implementation is operating
    //     // (< 10 in this case).

    //     // TODO: it's not perfect but enough for now
    //     let mut normalized = a;
    //     if normalized < 0.0 {
    //         positive = false;
    //         normalized = normalized.abs();
    //     }

    //     if normalized != 0.0 {
    //         while normalized >= 10.0 {
    //             normalized /= 10.0;
    //             exponent += 1;
    //         }

    //         while normalized < 1.0 {
    //             normalized *= 10.0;
    //             exponent -= 1;
    //         }
    //     }

    //     // Panic, if invalid number if given
    //     println!("normalized {}", normalized);

    //     let significand: u64 = format!("{}", normalized).replace('.', "").parse().unwrap();
    //     Self {
    //         positive,
    //         exponent,
    //         significand,
    //     }
    // }

    /// Convert to integer, exp is always negative, or 0, significand is integer
    // fn from(a: f64) -> Self {
    //     // TODO: exponent is always negative so we could store it as u32
    //     let mut exponent = 0;
    //     let mut positive = true;

    //     // TODO: it's not perfect but enough for now
    //     let mut normalized = a;

    //     println!("converting {}", a);

    //     if normalized < 0.0 {
    //         positive = false;
    //         normalized = normalized.abs();
    //     }

    //     if normalized > 0.0 {
    //         while normalized < 1.0 {
    //             normalized *= 10.0;
    //             exponent -= 1;
    //         }
    //     }

    //     if ! (normalized < 1.0) {
    //         let mut step_ahead = normalized * 10.0;

    //         while (step_ahead % 10.0) > 0.0 {
    //             println!("step_ahead {}, modulo {}, > 0? {}", step_ahead, step_ahead % 10.0, (step_ahead % 10.0) > 0.0);
    //             normalized = step_ahead;
    //             step_ahead *= 10.0;
    //             exponent -= 1;
    //         }

    //     }

    //     let significand: u64 = format!("{}", normalized).replace('.', "").parse().unwrap();

    //     let result = Self {
    //         positive,
    //         exponent,
    //         significand
    //     };

    //     println!("from float: {} {:?}", a, result);

    //     result
    // }

    fn from(a: f64) -> Self {
        // TODO: exponent is always negative so we could store it as u32
        let mut exponent = 0;
        let mut positive = true;

        // TODO: it's not perfect but enough for now
        let mut normalized = a;

        println!("converting {}", a);

        if normalized < 0.0 {
            positive = false;
            normalized = normalized.abs();
        }

        // Convert to string representation, stupid, and simple.
        // But that solves many issues when math is used to get decimal point position
        let normalized_str = format!("{}", normalized);

        println!("normalized {}", normalized);

        let parts: Vec<_> = normalized_str.split('.').collect();
        if parts.len() > 1 {
            // there is something after a '.'
            exponent = -1 * parts[1].len() as i32;
        }

        let significand: u64 = parts.join("").parse().unwrap();

        let result = Self {
            positive,
            exponent,
            significand,
        };

        println!("from float: {} {:?}", a, result);

        result
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
        let mut exponent = self.exponent + rhs.exponent;
        // TODO: figure out what happens in case of overflow
        let mut significand = self.significand * rhs.significand;

        if exponent < 0 {
            // Exponent < 0 -> there is a fraction part
            // Remove trailing zeros from fraction part
            while significand > 0 && significand % 10 == 0 {
                significand /= 10;
                exponent += 1;
            }
        }

        if significand == 0 {
            // normalize zero
            exponent = 0;
        }

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

            if self.exponent == other.exponent {
                // sign and exponend are the same, compare the value
                return Some(a.significand.cmp(&b.significand));
            }

            let exp = a.exponent.min(b.exponent).abs() as u32;
            // TODO: when switching to base of 2, that would be a simple bitshift
            let exp = 10_u64.pow(exp);

            // Get whole values
            let mut a_whole = a.significand / exp;
            let mut b_whole = b.significand / exp;

            if a_whole == 0 && b_whole == 0 {
                // both values < 1.0
                // compare exponents
                return Some(a.exponent.cmp(&b.exponent));
            }

            // One of values, or both are >= 1, compare
            return Some(a_whole.cmp(&b_whole));
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
    fn to_string_works() {
        for t in [100.0, 10.0, 1.0, 0.0, 0.1, 0.01, 0.001] {
            assert_eq!(format!("{}", SoftFloat::from(t)), format!("{}", t));
        }
    }

    #[test]
    fn convert_from_float_works() {
        assert_eq!(
            SoftFloat::from(100.0),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 100
            }
        );
        assert_eq!(
            SoftFloat::from(10.0),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 10
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

        assert_eq!(
            SoftFloat::from(12.5),
            SoftFloat {
                positive: true,
                exponent: -1,
                significand: 125
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

        // // Mixed numbers
        assert!(d < c);
        assert!(c > d);

        // // Negative numbers
        assert!(f == f);
        assert!(e < f);
        assert!(f > e);
        assert!(d < e);

        assert!(SoftFloat::from(11.01) > SoftFloat::from(11.001));
    }

    #[test]
    fn multiplication_works() {
        for (a, b, c) in [
            (25.0, 0.5, 12.5),
            (0.9, 0.1, 0.09),
            (3.14, 2.0, 6.28),
            (6.28, 0.02, 0.1256),
            (6.28, 0.5, 3.14),
            (0.00046, 0.000764, 0.00000035144),
            // Check for zero
            (1.0, 0.0, 0.0),
            (0.0, 0.1, 0.0),
            (12345413.054322345000004, 0.0, 0.0),
            // Check for one
            (1.0, 0.1, 0.1),
            (0.1, 1.0, 0.1),
            (0.01, 1.0, 0.01),
            (1.01, 1.0, 1.01),
            (10.0, 1.0, 10.0),
            (100.0, 1.0, 100.0),
            (10.01, 1.0, 10.01),
            // Check for base - 10 for now
            (10.0, 10.0, 100.0),
            (10.0, 100.0, 1000.0),
            (10.1, 10.01, 101.101),
            (10.0, 10.05, 100.5),
            (0.00001, 10000.0, 0.1),
        ] {
            assert_eq!(SoftFloat::from(a) * SoftFloat::from(b), SoftFloat::from(c));
        }
    }
}
