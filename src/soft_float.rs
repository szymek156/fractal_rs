//! Attemp to create a custom floating type for fractal

use std::cmp::Ordering;
use std::fmt;
use std::mem::swap;
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

impl SoftFloat {
    /// ```
    /// 3.1415000 -> 3.1415
    /// 1.0       -> 1.0
    /// ```
    pub fn remove_trailing_zeros(&mut self) {
        if self.exponent < 0 {
            // Exponent < 0 -> there is a fraction part
            // Remove trailing zeros from fraction part
            while self.significand > 0 && self.significand % 10 == 0 {
                self.significand /= 10;
                self.exponent += 1;
            }
        }

        if self.significand == 0 {
            // normalize zero
            self.exponent = 0;
            self.positive = true;
        }
    }

    /// ```
    /// 1.1234    -> 1.234000
    /// 1.0005646 -> 1.000546
    /// ```
    /// Note it changes significands, but does not changes exponents.
    /// Therefore after this conversion, representation is invalid!
    fn expand_significand_to(&mut self, b: &mut Self) {
        let a_exp = self.exponent.abs();
        let b_exp = b.exponent.abs();

        if a_exp < b_exp {
            let zeros = b_exp - a_exp;
            println!("a_exp {}, b_exp {}, zeros {}", a_exp, b_exp, zeros);
            self.significand *= 10u64.pow(zeros as u32);
        } else if a_exp > b_exp {
            let zeros = a_exp - b_exp;
            b.significand *= 10u64.pow(zeros as u32);
        }
    }
}

/// Get nice "float like" representation
impl fmt::Display for SoftFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut with_dec = self.significand.to_string();

        if self.exponent < 0 {
            // there is fraction part
            let idx = with_dec.len() as i32 + self.exponent;
            if idx <= 0 {
                // value is < 1.0, add leading zeros
                let zeros = format!("0.{}", "0".repeat(idx.abs() as usize));
                with_dec.insert_str(0, &zeros);
            } else {
                with_dec.insert(idx as usize, '.');
            }
        } // else, no fraction part, print 1, not 1.0

        write!(f, "{}{}", if self.positive { "" } else { "-" }, with_dec)
    }
}

impl From<f64> for SoftFloat {
    fn from(a: f64) -> Self {
        // TODO: exponent is always negative so we could store it as u32
        let mut exponent = 0;
        let mut positive = true;

        // TODO: it's not perfect but enough for now
        let mut normalized = a;

        //println!("converting {}", a);

        if normalized < 0.0 {
            positive = false;
            normalized = normalized.abs();
        } else if normalized == -0.0 {
            // Stupid corner case of floats
            normalized = 0.0;
        }

        // Convert to string representation, stupid, and simple.
        // But that solves many issues when math is used to get decimal point position
        let normalized_str = format!("{}", normalized);

        //println!("normalized {}", normalized);

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

        //println!("from float: {} {:?}", a, result);

        result
    }
}

impl Add for SoftFloat {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self {
        // TODO: overflowing?

        if self.positive && !rhs.positive {
            // 5 + (-3) = 5 - 3
            rhs.positive = true;

            return self - rhs;
        } else if !self.positive && rhs.positive {
            // -5 + 3 = 3 - +5
            // -20 + 30 = 30 - +20
            self.positive = true;

            return rhs - self;
        }

        // Both positive, or both negative
        self.expand_significand_to(&mut rhs);

        let significand = self.significand + rhs.significand;
        let exponent = self.exponent.min(rhs.exponent);
        let positive = self.positive && rhs.positive;

        let mut res = Self {
            positive,
            exponent,
            significand,
        };

        res.remove_trailing_zeros();

        res
    }
}

impl Sub for SoftFloat {
    type Output = Self;

    fn sub(mut self, mut rhs: Self) -> Self::Output {
        if !self.positive && rhs.positive {
            // -25.0 -  +0.5 -> 25 + 0.5 -> 25.5 -> -25.5
            self.positive = true;

            let mut res = self + rhs;
            res.positive = false;

            return res;
        } else if self.positive && !rhs.positive {
            rhs.positive = true;
            // +4 - -3 -> 4 + 3 = 7
            return self + rhs;
        }

        // 4 - 5
        // -25 - -0.5 = -25 + 0.5 = -24.5
        // Both positive, or negative
        self.expand_significand_to(&mut rhs);

        let mut significand = 0;
        let mut positive = true;

        if self.significand < rhs.significand {
            // 5 - 7 --> 7 - 5 -> 2 -> -2
            significand = rhs.significand - self.significand;

            positive = if !self.positive && !rhs.positive {
                // -25 - -30 =  -25 + 30 = 5
                true
            } else
            /* both positive, other cases are handled above */
            {
                false
            };
        } else {
            // 6 - 4 = 2
            significand = self.significand - rhs.significand;

            positive = if !self.positive && !rhs.positive {
                // -25 - -0.5 = -24.5
                false
            } else /* both positive, other cases handled at the begining */ {
                // +6 - +4 = 2
                true
            };
        }

        let exponent = self.exponent.min(rhs.exponent);

        let mut res = Self {
            positive,
            exponent,
            significand,
        };

        res.remove_trailing_zeros();

        res
    }
}

impl Mul for SoftFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // println!("Mul a {}, b {}", self, rhs);

        let positive = self.positive == rhs.positive;
        let mut exponent = self.exponent + rhs.exponent;
        // Will panic in case of overflow - in debug, in release returns some crap

        // try to multiply, if res > u64 max, divide by 10 until >, decrement exponent
        // TODO: this requires bigger buffer, is there a way, to avoid that?
        let mut tmp = self.significand as u128 * rhs.significand as u128;

        // TODO: maybe truncate to bigger exponent of those two?
        while tmp > u64::MAX as u128 {
            tmp /= 10;
            exponent += 1;
        }

        // println!("tmp {}", tmp);
        let mut significand = tmp as u64; 

        // println!("self.exponent {}, exponent {}", self.exponent, exponent);
        // println!("self.significand {} significand {}", self.significand, significand);

        let mut res = SoftFloat {
            positive,
            exponent,
            significand,
        };
    
        res.remove_trailing_zeros();

        res
    }
}

impl PartialOrd for SoftFloat {
    // TODO: this is ridicilous, approach needs to be changed
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.positive == other.positive {
            // The same sign
            let mut a = &self;
            let mut b = &other;

            if !self.positive {
                // Comparing negative numbers, changed order of "self" and "other" in cmp call
                b = &self;
                a = &other;
                // TODO: ?
                // swap(a,b);
            }

            let a_split = 10u64.pow(a.exponent.abs() as u32);
            if a_split == 0 {
                // TODO: Q&D, forget it
                return Some(Ordering::Less);
                // //println!("WTF: a={:?}, b={:?}", a, b);
                // //println!("a.exp.abs {}", a.exponent.abs());
                // //println!("a.exp.abs as u32 {}", a.exponent.abs() as u32);
                // // WHY THE FUCK it returns 0
                // //println!("10^exp {}", 10u64.pow(a.exponent.abs() as u32));
            }
            let a_whole = a.significand / a_split;
            let mut a_fraction = a.significand % a_split;
            // TODO: fix that later
            let a_fraction_len = format!("{}", a_fraction).len() as i32;
            let mut a_norm_exp = a.exponent;

            let b_split = 10u64.pow(b.exponent.abs() as u32);
            let b_whole = b.significand / b_split;
            let mut b_fraction = b.significand % b_split;
            let b_fraction_len = format!("{}", b_fraction).len() as i32;
            let mut b_norm_exp = b.exponent;

            //println!("a_fraction {} b_fraction {}", a_fraction, b_fraction);

            if a_fraction_len != b_fraction_len {
                let zeros = (a_fraction_len - b_fraction_len).abs() as u32;

                if a_fraction_len < b_fraction_len {
                    // TODO: this can explode
                    // 0.9 cmp 0.000000000000011
                    a_fraction *= 10u64.pow(zeros);
                    a_norm_exp -= zeros as i32;
                } else {
                    b_fraction *= 10u64.pow(zeros);
                    b_norm_exp -= zeros as i32;
                }
            }

            //println!("Comparing {:?} with {:?}", a, b);
            //println!("a_whole {} b_whole {}", a_whole, b_whole);
            //println!(
                // "a_fraction normalized {} b_fraction normalized {}",
                // a_fraction, b_fraction
            // );

            if a_whole == b_whole {
                //println!("a_whole == b_whole");
                if a_fraction == b_fraction {
                    //println!("a_fraction == b_fraction");
                    if a_norm_exp == b_norm_exp {
                        //println!("a.exponent == b.exponent");
                        return Some(Ordering::Equal);
                    } else {
                        //println!("a.exponent != b.exponent");
                        return Some(a.exponent.cmp(&b.exponent));
                    }
                } else if a_fraction < b_fraction {
                    //println!("a_fraction < b_fraction");
                    if a.exponent == b.exponent {
                        //println!("a.exponent == b.exponent");
                        return Some(a_fraction.cmp(&b_fraction));
                    } else {
                        //println!("a.exponent != b.exponent");
                        return Some(a.exponent.cmp(&b.exponent));
                    }
                } else
                /* fraction_a > fraction_b */
                {
                    //println!("a_fraction > b_fraction");
                    if a_norm_exp == b_norm_exp {
                        //println!("a.exponent == b.exponent");
                        return Some(a_fraction.cmp(&b_fraction));
                    } else {
                        //println!("a.exponent != b.exponent");
                        // here
                        return Some(a.exponent.cmp(&b.exponent));
                    }
                }
            } else {
                //println!("a_whole == b_whole");
                return Some(a_whole.cmp(&b_whole));
            }
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
            exponent: -7,
            significand: 125,
        };

        assert_eq!("0.0000125", format!("{}", sf));
    }

    #[test]
    fn to_string_exp_positive_works() {
        // 60.89523
        let sf = SoftFloat {
            positive: true,
            exponent: -5,
            significand: 6089523,
        };

        assert_eq!("60.89523", format!("{}", sf));
    }

    #[test]
    fn to_string_negative_number_works() {
        // 60.89523
        let sf = SoftFloat {
            positive: false,
            exponent: -5,
            significand: 6089523,
        };

        assert_eq!("-60.89523", format!("{}", sf));
    }

    #[test]
    fn to_string_works() {
        for t in [
            100.0, 10.0, 1.0, 1.0101, 0.0, 0.1, 0.01, 0.001, //
            -100.0, -10.0, -1.0, -1.0101, -0.1, -0.01, -0.001,
        ] {
            assert_eq!(format!("{}", SoftFloat::from(t)), format!("{}", t));
        }

        // There is no negative zero
        assert_eq!(format!("{}", SoftFloat::from(-0.0)), "0");
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
            SoftFloat::from(0.00),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 0
            }
        );
        assert_eq!(
            SoftFloat::from(0.0000000),
            SoftFloat {
                positive: true,
                exponent: 0,
                significand: 0
            }
        );

        assert_eq!(
            SoftFloat::from(-0.0),
            SoftFloat {
                // Don't accept negative zeros
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

    // TODO: add fuzz testing
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
        assert!(SoftFloat::from(0.09) > SoftFloat::from(0.011));
        assert!(SoftFloat::from(9.1234) > SoftFloat::from(9.123));

        assert!(SoftFloat::from(0.1) > SoftFloat::from(0.000123));

        // Comparsion turned out to be harder task than initially thought, there are many combinations
        // of sign, whole part, and fractional + exponent part.
        // This test tries to cover it all

        // sign | whole | fraction | exp
        // ++   |  ==   |    ==    | ==
        assert!(SoftFloat::from(1.23) == SoftFloat::from(1.23));
        // ++   |  ==   |    ==    | <
        assert!(SoftFloat::from(1.0023) < SoftFloat::from(1.23));
        // ++   |  ==   |    ==    | >
        assert!(SoftFloat::from(1.23) > SoftFloat::from(1.023));

        // ++   |  ==   |    <    | =
        assert!(SoftFloat::from(1.11) < SoftFloat::from(1.99));
        // ++   |  ==   |    <    | <
        assert!(SoftFloat::from(1.011) < SoftFloat::from(1.99));
        // ++   |  ==   |    <    | >
        assert!(SoftFloat::from(1.11) > SoftFloat::from(1.099));

        // ++   |  ==   |    >    | ==
        assert!(SoftFloat::from(1.99) > SoftFloat::from(1.11));
        // ++   |  ==   |    >    | <
        assert!(SoftFloat::from(1.099) < SoftFloat::from(1.11));
        // ++   |  ==   |    >    | >
        assert!(SoftFloat::from(1.99) > SoftFloat::from(1.011));

        // ++   |  <   |    ==    | ==
        assert!(SoftFloat::from(2.0) < SoftFloat::from(3.0));
        // TODO: add 0
    }

    #[test]
    fn multiplication_works() {
        for (a, b, c) in [(0.00001, 10000.0, 0.1)] {
            assert_eq!(SoftFloat::from(a) * SoftFloat::from(b), SoftFloat::from(c));
        }
    }

    #[test]
    fn addition_works() {
        for (a, b, c) in [
            // Both positive
            (25.0, 0.5, 25.5),
            (0.9, 0.1, 1.0),
            (3.14, 2.0, 5.14),
            (6.28, 0.02, 6.3),
            (6.28, 0.5, 6.78),
            (0.00046, 0.000764, 0.001224),
            (0.0, 0.1, 0.1),
            (5.013, 0.0, 5.013),
            (0.0, 0.0, 0.0),
            (12345413.0543223, 0.0, 12345413.0543223),
            (0.1, 1.0, 1.1),
            (0.01, 1.0, 1.01),
            (1.01, 1.0, 2.01),
            (10.0, 1.0, 11.0),
            (100.0, 1.0, 101.0),
            (10.01, 1.0, 11.01),
            (10.0, 10.0, 20.0),
            (10.0, 100.0, 110.0),
            (10.1, 10.01, 20.11),
            (0.00001, 10000.0, 10000.00001),
            // First negative
            (-25.0, 0.5, -24.5),
            (-0.9, 0.1, -0.8),
            (-3.14, 2.0, -1.14),
            (-6.28, 0.02, -6.26),
            (-6.28, 0.5, -5.78),
            (-0.00046, 0.000764, 0.000304),
            (-0.0, 0.1, 0.1),
            (-5.013, 0.0, -5.013),
            (-0.0, 0.0, 0.0),
            (-12345413.0543223, 0.0, -12345413.0543223),
            (-0.1, 1.0, 0.9),
            (-0.01, 1.0, 0.99),
            (-1.01, 1.0, -0.01),
            (-10.0, 1.0, -9.0),
            (-100.0, 1.0, -99.0),
            (-10.01, 1.0, -9.01),
            (-10.0, 10.0, 0.0),
            (-10.0, 100.0, 90.0),
            (-10.1, 10.01, -0.09),
            (-0.00001, 10000.0, 9999.99999),
            // Second negative
            (25.0, -0.5, 24.5),
            (0.9, -0.1, 0.8),
            (3.14, -2.0, 1.14),
            (6.28, -0.02, 6.26),
            (6.28, -0.5, 5.78),
            (0.00046, -0.000764, -0.000304),
            (0.0, -0.1, -0.1),
            (5.013, -0.0, 5.013),
            (0.0, -0.0, 0.0),
            (12345413.0543223, -0.0, 12345413.0543223),
            (0.1, -1.0, -0.9),
            (0.01, -1.0, -0.99),
            (1.01, -1.0, 0.01),
            (10.0, -1.0, 9.0),
            (100.0, -1.0, 99.0),
            (10.01, -1.0, 9.01),
            (10.0, -10.0, 0.0),
            (10.0, -100.0, -90.0),
            (10.1, -10.01, 0.09),
            (0.00001, -10000.0, -9999.99999),
            // Both negative
            (-25.0, -0.5, -25.5),
            (-0.9, -0.1, -1.0),
            (-3.14, -2.0, -5.14),
            (-6.28, -0.02, -6.3),
            (-6.28, -0.5, -6.78),
            (-0.00046, -0.000764, -0.001224),
            (-0.0, -0.1, -0.1),
            (-5.013, -0.0, -5.013),
            (-0.0, -0.0, -0.0),
            (-12345413.0543223, -0.0, -12345413.0543223),
            (-0.1, -1.0, -1.1),
            (-0.01, -1.0, -1.01),
            (-1.01, -1.0, -2.01),
            (-10.0, -1.0, -11.0),
            (-100.0, -1.0, -101.0),
            (-10.01, -1.0, -11.01),
            (-10.0, -10.0, -20.0),
            (-10.0, -100.0, -110.0),
            (-10.1, -10.01, -20.11),
            (-0.00001, -10000.0, -10000.00001),
        ] {
            //println!("{} + {} = {}", a, b, c);
            assert_eq!(SoftFloat::from(a) + SoftFloat::from(b), SoftFloat::from(c));
        }
    }

    #[test]
    fn substraction_works() {
        for (a, b, c) in [
            // Both positive
            // First negative
            (25.0, 0.5, 24.50),
            (0.9, 0.1, 0.80),
            (3.14, 2.0, 1.14),
            (6.28, 0.02, 6.26),
            (6.28, 0.5, 5.78),
            (0.00046, 0.000764, -0.000304),
            (0.0, 0.1, -0.10),
            (5.013, 0.0, 5.013),
            (0.0, 0.0, 0.00),
            (12345413.0543223, 0.0, 12345413.0543223),
            (0.1, 1.0, -0.90),
            (0.01, 1.0, -0.99),
            (1.01, 1.0, 0.01),
            (10.0, 1.0, 9.00),
            (100.0, 1.0, 99.00),
            (10.01, 1.0, 9.01),
            (10.0, 10.0, 0.00),
            (10.0, 100.0, -90.00),
            (10.1, 10.01, 0.09),
            (0.00001, 10000.0, -9999.99999),
            (-25.0, 0.5, -25.5),
            (-0.9, 0.1, -1.0),
            (-3.14, 2.0, -5.14),
            (-6.28, 0.02, -6.30),
            (-6.28, 0.5, -6.78),
            // (-0.00046, 0.000764, 0.0),
            // (-0.0, 0.1, 0.00),
            // (-5.013, 0.0, -5.01),
            // (-0.0, 0.0, 0.00),
            // (-12345413.0543223, 0.0, -12345413.0543223),
            // (-0.1, 1.0, -1.10),
            // (-0.01, 1.0, -1.01),
            // (-1.01, 1.0, -2.01),
            // (-10.0, 1.0, -11.00),
            // (-100.0, 1.0, -101.00),
            // (-10.01, 1.0, -11.01),
            // (-10.0, 10.0, -20.00),
            // (-10.0, 100.0, -110.00),
            (-10.1, 10.01, -20.11),
            (-0.00001, 10000.0, -10000.00001),
            (25.0, -0.5, 25.50),
            (0.9, -0.1, 1.00),
            (3.14, -2.0, 5.14),
            (6.28, -0.02, 6.30),
            (6.28, -0.5, 6.78),
            (0.00046, -0.000764, 0.001224),
            (0.0, -0.1, 0.10),
            (5.013, -0.0, 5.013),
            (0.0, -0.0, 0.00),
            (12345413.0543223, -0.0, 12345413.0543223),
            (0.1, -1.0, 1.10),
            (0.01, -1.0, 1.01),
            (1.01, -1.0, 2.01),
            (10.0, -1.0, 11.00),
            (100.0, -1.0, 101.00),
            (10.01, -1.0, 11.01),
            (10.0, -10.0, 20.00),
            (10.0, -100.0, 110.00),
            (10.1, -10.01, 20.11),
            (0.00001, -10000.0, 10000.00001),
            (-25.0, -0.5, -24.50),
            (-0.9, -0.1, -0.80),
            (-3.14, -2.0, -1.14),
            (-6.28, -0.02, -6.26),
            (-6.28, -0.5, -5.78),
            (-0.00046, -0.000764, 0.000304),
            (-0.0, -0.1, 0.1),
            (-5.013, -0.0, -5.013),
            (-0.0, -0.0, 0.00),
            (-12345413.0543223, -0.0, -12345413.0543223),
            (-0.1, -1.0, 0.90),
            (-0.01, -1.0, 0.99),
            (-1.01, -1.0, -0.01),
            (-10.0, -1.0, -9.00),
            (-100.0, -1.0, -99.00),
            (-10.01, -1.0, -9.01),
            (-10.0, -10.0, 0.00),
            (-10.0, -100.0, 90.00),
            (-10.1, -10.01, -0.09),
            (-0.00001, -10000.0, 9999.99999),
            // Second negative
            // Both negative
        ] {
            //println!(" TESTING {} - {} = {}", a, b, c);
            assert_eq!(SoftFloat::from(a) - SoftFloat::from(b), SoftFloat::from(c));
        }
    }

    #[test]
    fn fixing_fractal_mul_overflow() {
        let x = SoftFloat::from(-0.7436438870371587);
        println!("x = {}", x);
        let x2 = SoftFloat::from(0.5530062307277344);
        println!("x2 = {}", x2);

        assert_eq!(x*x, x2);       
    }

    #[test]
    fn fixing_fractal_expand_overflow() {
        //x2 0.5530062307277344492
        let x2 = SoftFloat {positive: true, exponent: -19, significand: 5530062307277344492};

        //y2 0.017378069019548090773
        let y2 = SoftFloat {positive: true, exponent: -21, significand: 17378069019548090773};

        let sum = SoftFloat::from(0.5703842997472824);

        assert_eq!(x2 + y2, sum);
    }

}
