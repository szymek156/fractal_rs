//! This one is the actual attempt to implement IEEE 754
//! https://ciechanow.ski/exposing-floating-point/

use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};
use std::{fmt, mem};

use rug::ops::Pow;

// Lets try binary32 first.
// exp range [-126; 127]
const BIAS: u32 = 127;
const EXPONENT_WIDTH: u32 = 8;
const EXPONENT_MASK: u32 = 0xFF;
// 24, with implicit bit
const SIGNIFICAND_WIDTH: u32 = 23;
const SIGNIFICAND_MASK: u32 = 0x7FFFFF;

// clone + copy to be able to do: x + x etc.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct SoftFloat {
    negative: bool,
    exponent: u32,
    significand: u64,
}

impl SoftFloat {
    /// When the biased exponent is set to 0, the exponent is interpreted as −126
    /// (not −127 despite the bias), and the leading digit is assumed to be 0.
    pub fn is_subnormal(&self) -> bool {
        self.exponent == 0 && self.significand != 0
    }

    /// A float number with maximum biased exponent value and all zeros in significand
    /// is interpreted as positive or negative infinity depending on the value of the sign bit
    pub fn is_pos_infinity(&self) -> bool {
        !self.negative && self.exponent == EXPONENT_MASK && self.significand == 0
    }

    pub fn is_neg_infinity(&self) -> bool {
        self.negative && self.exponent == EXPONENT_MASK && self.significand == 0
    }

    pub fn is_neg_zero(&self) -> bool {
        self.negative && self.exponent == 0 && self.significand == 0
    }

    pub fn is_pos_zero(&self) -> bool {
        !self.negative && self.exponent == 0 && self.significand == 0
    }

    ///  A float number with maximum biased exponent value and non-zero
    /// significand is interpreted as NaN – Not a Number:
    pub fn is_nan(&self) -> bool {
        self.exponent == EXPONENT_MASK && self.significand != 0
    }

    /// Gets unbiased exponent
    pub fn get_exponent(&self) -> i32 {
        if self.is_subnormal() {
            return -(BIAS as i32)  + 1;
        }

        self.exponent as i32 - BIAS as i32
    }

    pub fn pos_inf() -> Self {
        Self {
            negative: false,
            exponent: EXPONENT_MASK,
            significand: 0,
        }
    }

    pub fn neg_inf() -> Self {
        Self {
            negative: true,
            exponent: EXPONENT_MASK,
            significand: 0,
        }
    }

    pub fn normalize(&mut self) {

        println!("normalizing {:?}", self);

        if self.is_nan() || self.is_neg_infinity() || self.is_pos_infinity() {
            // Nothing to normalize
            return;
        }

        if self.is_subnormal() {
            todo!()
        }

        // If there is anything outside 23 bits range - shift it
        // 1 is the implicit bit on 24th position
        // TODO: hope compiler hoists those values
        while (self.significand & !(SIGNIFICAND_MASK as u64)) > (1 << SIGNIFICAND_WIDTH) {
            // Does this overflow?
            self.significand >>= 1;
            self.exponent += 1;
            
        }

        // Make bit on 24th position implicit
        self.significand &= SIGNIFICAND_MASK as u64;
        // TODO: round to nearest, half to even

    }

    // TODO: Handle NaNs
    // The easiest way to obtain NaN directly is by using NAN macro.
    // In practice though, NaN arises in the following set of operations:

    // ±0.0 multiplied by ±infinity
    // −infinity added to +infinity
    // ±0.0 divided by ±0.0
    // ±infinity divided by ±infinity
    // square root of a negative number (−0.0 is fine though!)

    // By default the result of any operation involving NaNs will result in a NaN as well.
    // That’s one of the reasons why compiler can’t optimize seemingly simple cases like
    // a + (b - b) into just a. If b is NaN the result of the entire operation has to be NaN too.

    // NaNs are not equal to anything, even to themselves. If you were to look at your compiler’s
    // implementation of isnan function you’d see something like return x != x;.
}

/// Get nice "float like" representation
impl fmt::Display for SoftFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For now, go the easiest way, while expanding the resolution,
        // this might be pretty tricky to implement.

        let mut bits: u32 = 0;

        bits = (self.negative as u32) << (EXPONENT_WIDTH + SIGNIFICAND_WIDTH)
            | (self.exponent << SIGNIFICAND_WIDTH)
            | (self.significand & SIGNIFICAND_MASK as u64) as u32;

        write!(f, "{}", f32::from_bits(bits))
    }
}

impl From<f32> for SoftFloat {
    fn from(a: f32) -> Self {
        let bits = a.to_bits();

        let negative = (bits >> (EXPONENT_WIDTH + SIGNIFICAND_WIDTH)) == 1;
        let exponent = (bits >> (SIGNIFICAND_WIDTH)) & EXPONENT_MASK;
        let significand = (bits & SIGNIFICAND_MASK) as u64;

        Self {
            negative,
            exponent,
            significand,
        }
    }
}

impl From<f64> for SoftFloat {
    fn from(a: f64) -> Self {
        todo!();
    }
}

impl Add for SoftFloat {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self {
        todo!();
    }
}

impl Sub for SoftFloat {
    type Output = Self;

    fn sub(mut self, mut rhs: Self) -> Self::Output {
        todo!();
    }
}

impl Mul for SoftFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let negative = self.negative ^ rhs.negative;
        // TODO: checked overflow may become handy
        // a_exp + b_exp = a_exp - bias + b_exp - bias = a_exp + b_exp - 2bias
        // => a_exp + b_exp -2bias + bias == added real exponents and biased
        let exponent = self.exponent + rhs.exponent - BIAS;

        if exponent > EXPONENT_MASK {
            if negative {
                return SoftFloat::neg_inf();
            } else {
                return SoftFloat::pos_inf();
            }
        }

        // TODO: do biasing?
        // 4 + 5 = 4 - 127 + 5 - 127 = 9 - 254 = -245
        // 
        let mut a_sig = self.significand;
        if !self.is_subnormal() {
            a_sig |= 1 << SIGNIFICAND_WIDTH;
        }

        // TODO: make sure mul of mixed subnormal and normal works
        let mut b_sig = rhs.significand;
        if !rhs.is_subnormal() {
            b_sig |= 1 << SIGNIFICAND_WIDTH;
        }

        // As for now (32bit float implementation) u64 buffer is more than
        // required to keep the resulting value.
        // 0 in significand with implicit 1 is 8388608 in binary, 1 << 23,
        // so two smallest values will ever be multiplied is 1 << 23 * 1 << 23, 
        // that results with 1 << 46. Shift back by width of significand ( >> 23)
        // To have bits in correct place, from that place normalization can be done.
        let significand = (a_sig * b_sig) >> SIGNIFICAND_WIDTH;

        // TODO: normalize result
        let mut res = SoftFloat {
            negative,
            exponent,
            significand,
        };

        res.normalize();


        res
    }
}

impl PartialOrd for SoftFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // TODO: check for submormals, inf's, nan's
        // Numbers are in normalized from, should be easy, eh?!
        if self.negative == other.negative {
            // Both positive, or negative

            let mut a = self;
            let mut b = other;

            if self.negative {
                a = other;
                b = self;
            }

            if a.exponent == b.exponent {
                return Some(a.significand.cmp(&b.significand));
            }

            return Some(a.exponent.cmp(&b.exponent));
        } else {
            if self.negative {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Greater);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string_exp_negative_works() {
        // // 0.0000125
        // let sf = SoftFloat {
        //     positive: true,
        //     exponent: -7,
        //     significand: 125,
        // };

        // assert_eq!("0.0000125", format!("{}", sf));
    }

    #[test]
    fn to_string_exp_positive_works() {
        // // 60.89523
        // let sf = SoftFloat {
        //     positive: true,
        //     exponent: -5,
        //     significand: 6089523,
        // };

        // assert_eq!("60.89523", format!("{}", sf));
    }

    #[test]
    fn to_string_negative_number_works() {
        // // 60.89523
        // let sf = SoftFloat {
        //     positive: false,
        //     exponent: -5,
        //     significand: 6089523,
        // };

        // assert_eq!("-60.89523", format!("{}", sf));
    }

    #[test]
    fn to_string_works() {
        for t in [
            100.0f32, 10.0f32, 1.0f32, 1.0101f32, 0.0f32, 0.1f32, 0.01f32, 0.001f32, //
            -100.0f32, -10.0f32, -1.0f32, -1.0101f32, -0.1f32, -0.01f32, -0.001f32,
        ] {
            assert_eq!(format!("{}", SoftFloat::from(t)), format!("{}", t));
        }

        assert_eq!(format!("{}", SoftFloat::from(-0.0f32)), "-0");
    }

    #[test]
    fn convert_from_float_works() {
        // assert_eq!(
        //     SoftFloat::from(100.0),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 100
        //     }
        // );
        // assert_eq!(
        //     SoftFloat::from(10.0),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 10
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(1.0),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 1
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(0.0),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 0
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(0.00),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 0
        //     }
        // );
        // assert_eq!(
        //     SoftFloat::from(0.0000000),
        //     SoftFloat {
        //         positive: true,
        //         exponent: 0,
        //         significand: 0
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(-0.0),
        //     SoftFloat {
        //         // Don't accept negative zeros
        //         positive: true,
        //         exponent: 0,
        //         significand: 0
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(0.1),
        //     SoftFloat {
        //         positive: true,
        //         exponent: -1,
        //         significand: 1
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(0.01),
        //     SoftFloat {
        //         positive: true,
        //         exponent: -2,
        //         significand: 1
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(0.001),
        //     SoftFloat {
        //         positive: true,
        //         exponent: -3,
        //         significand: 1
        //     }
        // );

        // assert_eq!(
        //     SoftFloat::from(12.5),
        //     SoftFloat {
        //         positive: true,
        //         exponent: -1,
        //         significand: 125
        //     }
        // );
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
        let a = SoftFloat::from(100.2345f32);
        let b = SoftFloat::from(100.2346f32);
        let c = SoftFloat::from(1.2346f32);
        let d = SoftFloat::from(-99999.99999f32);
        let e = SoftFloat::from(-99999.99998f32);
        let f = SoftFloat::from(-0.098f32);

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
        // TODO: this falls down to -10000.0 in both cases
        // assert!(d < e);

        assert!(SoftFloat::from(11.01f32) > SoftFloat::from(11.001f32));
        assert!(SoftFloat::from(0.09f32) > SoftFloat::from(0.011f32));
        assert!(SoftFloat::from(9.1234f32) > SoftFloat::from(9.123f32));

        assert!(SoftFloat::from(0.1f32) > SoftFloat::from(0.000123f32));

        // Comparsion turned out to be harder task than initially thought, there are many combinations
        // of sign, whole part, and fractional + exponent part.
        // This test tries to cover it all

        // sign | whole | fraction | exp
        // ++   |  ==   |    ==    | ==
        assert!(SoftFloat::from(1.23f32) == SoftFloat::from(1.23f32));
        // ++   |  ==   |    ==    | <
        assert!(SoftFloat::from(1.0023f32) < SoftFloat::from(1.23f32));
        // ++   |  ==   |    ==    | >
        assert!(SoftFloat::from(1.23f32) > SoftFloat::from(1.023f32));

        // ++   |  ==   |    <    | =
        assert!(SoftFloat::from(1.11f32) < SoftFloat::from(1.99f32));
        // ++   |  ==   |    <    | <
        assert!(SoftFloat::from(1.011f32) < SoftFloat::from(1.99f32));
        // ++   |  ==   |    <    | >
        assert!(SoftFloat::from(1.11f32) > SoftFloat::from(1.099f32));

        // ++   |  ==   |    >    | ==
        assert!(SoftFloat::from(1.99f32) > SoftFloat::from(1.11f32));
        // ++   |  ==   |    >    | <
        assert!(SoftFloat::from(1.099f32) < SoftFloat::from(1.11f32));
        // ++   |  ==   |    >    | >
        assert!(SoftFloat::from(1.99f32) > SoftFloat::from(1.011f32));

        // ++   |  <   |    ==    | ==
        assert!(SoftFloat::from(2.0f32) < SoftFloat::from(3.0f32));
        // TODO: add 0
    }

    #[test]
    fn multiplication_works() {
        for (a, b, c) in [(0.00001f32, 10000.0f32, 0.1f32)] {
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
            // println!("{} + {} = {}", a, b, c);
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
}
