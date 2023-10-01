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
            intrinsics::assume(result < 1 << 4);
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
            intrinsics::assume(result <= u8::MAX as Self);
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
            intrinsics::assume(result <= u16::MAX as Self);
        }

        result
    }
}

impl SignedIsqrt for i64 {
    // Uses technique from
    // https://web.archive.org/web/20220118185505/https://www.codecodex.com/wiki/Calculate_an_integer_square_root#Java
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            // `self as u64 as f64` is faster than `self as f64`.
            let mut result = (self as u64 as f64).sqrt() as Self;
            if result * result > self {
                result -= 1;
            }

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
    // Uses technique from
    // https://web.archive.org/web/20220118185505/https://www.codecodex.com/wiki/Calculate_an_integer_square_root#Java
    fn isqrt(self) -> Self {
        let mut result = (self as f64).sqrt() as Self;
        if result
            .checked_mul(result)
            .map(|result_squared| result_squared > self)
            .unwrap_or(true)
        {
            result -= 1;
        }

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result <= u32::MAX as Self);
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

        let leading_zeros = self.leading_zeros();
        let result = if leading_zeros >= 64 {
            (self as u64).isqrt() as Self
        } else {
            // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
            let precondition_shift = leading_zeros & 0b111110;
            self <<= precondition_shift;

            let hi = (self >> 64) as u64;
            let lo = self as u64 as Self;

            let s_prime = hi.isqrt();
            let r_prime = hi - s_prime * s_prime;

            let numerator = ((r_prime as Self) << 32) | (lo >> 32);
            let denominator = (s_prime as Self) << 1;

            let q = numerator / denominator;
            let u = numerator % denominator;

            let mut s = (s_prime << 32) as Self + q;
            if (u << 32) | (lo as u32 as Self) < q * q {
                s -= 1;
            }
            s >> (precondition_shift >> 1)
        };

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result <= u64::MAX as Self);
        }

        result
    }
}
