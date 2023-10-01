use core::intrinsics;

pub trait SignedIsqrt: Sized {
    fn checked_isqrt(self) -> Option<Self>;
    fn isqrt(self) -> Self;
}
pub trait UnsignedIsqrt {
    fn isqrt(self) -> Self;
}

impl SignedIsqrt for i8 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = (self as f32).sqrt() as Self;

            // SAFETY: the result is nonnegative and less than or equal to `i8::MAX.isqrt()`.
            // Inform the optimizer about it.
            unsafe {
                intrinsics::assume(0 <= result);
                intrinsics::assume(result <= 11);
            }

            result
        })
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u8 {
    fn isqrt(self) -> Self {
        let result = (self as f32).sqrt() as Self;

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << ((Self::BITS as Self) >> 1));
        }

        result
    }
}

impl SignedIsqrt for i16 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = (self as f32).sqrt() as Self;

            // SAFETY: the result is nonnegative and less than or equal to `i16::MAX.isqrt()`.
            // Inform the optimizer about it.
            unsafe {
                intrinsics::assume(0 <= result);
                intrinsics::assume(result <= 181);
            }

            result
        })
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u16 {
    fn isqrt(self) -> Self {
        let result = (self as f32).sqrt() as Self;

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << ((Self::BITS as Self) >> 1));
        }

        result
    }
}

impl SignedIsqrt for i32 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = (self as f64).sqrt() as Self;

            // SAFETY: the result is nonnegative and less than or equal to `i32::MAX.isqrt()`.
            // Inform the optimizer about it.
            unsafe {
                intrinsics::assume(0 <= result);
                intrinsics::assume(result <= 46_340);
            }

            result
        })
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u32 {
    fn isqrt(self) -> Self {
        let result = (self as f64).sqrt() as Self;

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << ((Self::BITS as Self) >> 1));
        }

        result
    }
}

impl SignedIsqrt for i64 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = (self as u64).isqrt() as i64;

            // SAFETY: the result is nonnegative and less than or equal to `i64::MAX.isqrt()`.
            // Inform the optimizer about it.
            unsafe {
                intrinsics::assume(0 <= result);
                intrinsics::assume(result <= 3_037_000_499);
            }

            result
        })
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u64 {
    fn isqrt(self) -> Self {
        // This proof of correctness is a corrected version of the flawed proof at
        // https://web.archive.org/web/20220118185505/https://www.codecodex.com/wiki/Calculate_an_integer_square_root#Java
        //
        // One way to solve this problem is to find the perfect square at or just below the input and to use its square
        // root.
        //
        // `f64` has a 53-bit mantissa. Each input above 2^53 will convert to a representative `f64`. Inputs that
        // correspond to the same representative form a consecutive range of inputs.
        //
        // If there is no perfect square in that range of inputs, all inputs in the range will be above the desired
        // perfect square, and so flooring the representative's square root will give us the proper result.
        //
        // If there is a perfect square in that range of inputs and the representative is at or above the perfect
        // square, the floor of the representative's square root will be correct for all inputs that are at or above
        // the perfect square. The floor of the representative's square root minus one will be correct for the
        // remaining inputs.
        //
        // If there is a perfect square in that range of inputs and the representative is below the perfect square, the
        // floor of the representative's square root plus one will be correct for all inputs that are at or above the
        // perfect square. The floor of the representative's square root will be correct for the remaining inputs.
        //
        // There cannot be more than one perfect square in a range of inputs because the distance between nearby
        // perfect squares is much larger than the number of inputs in a range. The largest ranges of inputs appear
        // near `u64::MAX`, where the ranges contain about 2^(64 - 53) = 2^11 inputs. The smallest distance between
        // perfect squares after 2^53 is about (2^27 + 1)^2 - (2^27)^2 = 2^28 - 1. Since the smallest distance between
        // perfect squares after inaccuracies appear is always much larger than the size of the largest range of
        // inputs, there cannot be more than one perfect square in a range of inputs.
        //
        // Thus, the correct output will be the floor of the representative's square root plus -1, 0, or 1.

        let result = (self as f64).sqrt() as Self;
        let result = match result.checked_mul(result) {
            None => result - 1,
            Some(result_squared) if self < result_squared => result - 1,
            _ => {
                let result_plus_one = result + 1;
                match result_plus_one.checked_mul(result_plus_one) {
                    Some(result_plus_one_squared) if result_plus_one_squared <= self => {
                        result_plus_one
                    }
                    _ => result,
                }
            }
        };

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << ((Self::BITS as Self) >> 1));
        }

        result
    }
}

impl SignedIsqrt for i128 {
    #[inline]
    fn checked_isqrt(self) -> Option<Self> {
        if self < 0 {
            None
        } else {
            let result = (self as u128).isqrt() as Self;

            // SAFETY: the result is nonnegative and less than or equal to `i128::MAX.isqrt()`.
            // Inform the optimizer about it.
            unsafe {
                intrinsics::assume(0 <= result);
                intrinsics::assume(result <= 13_043_817_825_332_782_212);
            }

            Some(result)
        }
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u128 {
    fn isqrt(mut self) -> Self {
        // Performs a Karatsuba square root.
        // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

        type HalfBitsT = u64;

        const HALF_BITS: u32 = HalfBitsT::BITS;
        const QUARTER_BITS: u32 = HalfBitsT::BITS >> 1;

        let leading_zeros = self.leading_zeros();
        let result = if leading_zeros >= HALF_BITS {
            (self as HalfBitsT).isqrt() as Self
        } else {
            // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
            let precondition_shift = leading_zeros & (HALF_BITS - 2);
            self <<= precondition_shift;

            let hi = (self >> HALF_BITS) as HalfBitsT;
            let lo = self & (HalfBitsT::MAX as Self);

            let s_prime = hi.isqrt();
            let r_prime = hi - s_prime * s_prime;

            let numerator = ((r_prime as Self) << QUARTER_BITS) | (lo >> QUARTER_BITS);
            let denominator = (s_prime as Self) << 1;

            let q = numerator / denominator;
            let u = numerator % denominator;

            let mut s = (s_prime << QUARTER_BITS) as Self + q;
            if (u << QUARTER_BITS) | (lo & ((1 << QUARTER_BITS) - 1)) < q * q {
                s -= 1;
            }
            s >> (precondition_shift >> 1)
        };

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << ((Self::BITS as Self) >> 1));
        }

        result
    }
}
