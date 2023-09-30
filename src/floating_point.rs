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
        (self >= 0).then(|| (self as f32).sqrt() as Self)
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u8 {
    fn isqrt(self) -> Self {
        (self as f32).sqrt() as Self
    }
}

impl SignedIsqrt for i16 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| (self as f32).sqrt() as Self)
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u16 {
    fn isqrt(self) -> Self {
        (self as f32).sqrt() as Self
    }
}

impl SignedIsqrt for i32 {
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| (self as f64).sqrt() as Self)
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u32 {
    fn isqrt(self) -> Self {
        (self as f64).sqrt() as Self
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

        result
    }
}

impl SignedIsqrt for i128 {
    #[inline]
    fn checked_isqrt(self) -> Option<Self> {
        if self < 0 {
            None
        } else {
            Some((self as u128).isqrt() as Self)
        }
    }

    #[inline]
    fn isqrt(self) -> Self {
        self.checked_isqrt()
            .expect("argument of integer square root must be non-negative")
    }
}

impl UnsignedIsqrt for u128 {
    // Uses technique from
    // https://web.archive.org/web/20220118185505/https://www.codecodex.com/wiki/Calculate_an_integer_square_root#Java
    fn isqrt(self) -> Self {
        let leading_zeros = self.leading_zeros();
        let result = if leading_zeros >= 64 {
            (self as u64).isqrt() as Self
        } else {
            let mut bit_index = (65 - leading_zeros) & 0b1111110;
            let mut result = ((self >> bit_index) as u64).isqrt() as Self;
            bit_index >>= 1;
            result <<= bit_index;
            let mut bit = 1 << bit_index;
            while bit > 0 {
                bit >>= 1;
                let result_high = result | bit;
                if result_high * result_high <= self {
                    result = result_high;
                }
            }
            result
        };

        result
    }
}
