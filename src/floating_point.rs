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

            // SAFETY: the result is nonnegative and less than or equal to `i64::MAX.isqrt()`.
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
            intrinsics::assume(result < 1 << 8);
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
            intrinsics::assume(result < 1 << 16);
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
            intrinsics::assume(result < 1 << 32);
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
        // The algorithm is based on the one presented in
        // <https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Binary_numeral_system_(base_2)>
        // which cites as source the following C code:
        // <https://web.archive.org/web/20120306040058/http://medialab.freaknet.org/martin/src/sqrt/sqrt.c>.

        let leading_zeros = self.leading_zeros();
        let result = if leading_zeros >= 64 {
            (self as u64).isqrt() as Self
        } else {
            let even_bit_index = (65 - leading_zeros) & 0b1111110;

            let mut result = ((self >> even_bit_index) as u64).isqrt() as Self;
            self -= (result * result) << even_bit_index;
            result <<= even_bit_index;
            let mut even_bit = 1 << (even_bit_index - 2);

            while even_bit != 0 {
                let result_high = result | even_bit;
                if self >= result_high {
                    self -= result_high;
                    result = (result >> 1) | even_bit;
                } else {
                    result >>= 1;
                }
                even_bit >>= 2;
            }

            result
        };

        // SAFETY: the result fits in an integer with half as many bits.
        // Inform the optimizer about it.
        unsafe {
            intrinsics::assume(result < 1 << 64);
        }

        result
    }
}
