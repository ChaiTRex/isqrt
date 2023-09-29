use core::intrinsics;

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
    // https://optimi.wordpress.com/2010/12/02/how-to-compute-64-bit-integer-square-roots-very-quickly/
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
    // https://optimi.wordpress.com/2010/12/02/how-to-compute-64-bit-integer-square-roots-very-quickly/
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

pub trait SignedIsqrt: Sized {
    fn checked_isqrt(self) -> Option<Self>;
    fn isqrt(self) -> Self;
}

macro_rules! signed_isqrt {
    ($type:ty, $unsigned_type:ty) => {
        impl SignedIsqrt for $type {
            #[inline]
            fn checked_isqrt(self) -> Option<Self> {
                if self < 0 {
                    None
                } else {
                    Some((self as $unsigned_type).isqrt() as $type)
                }
            }

            #[inline]
            fn isqrt(self) -> Self {
                self.checked_isqrt()
                    .expect("argument of integer square root must be non-negative")
            }
        }
    };
}

signed_isqrt!(i128, u128);

pub trait UnsignedIsqrt {
    fn isqrt(self) -> Self;
}

macro_rules! unsigned_isqrt {
    ($unsigned_type:ty) => {
        impl UnsignedIsqrt for $unsigned_type {
            #[inline]
            fn isqrt(self) -> Self {
                if self < 2 {
                    return self;
                }

                // The algorithm is based on the one presented in
                // <https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Binary_numeral_system_(base_2)>
                // which cites as source the following C code:
                // <https://web.archive.org/web/20120306040058/http://medialab.freaknet.org/martin/src/sqrt/sqrt.c>.

                let mut op = self;
                let mut res = 0;
                let mut one = 1 << (self.ilog2() & !1);

                while one != 0 {
                    if op >= res + one {
                        op -= res + one;
                        res = (res >> 1) + one;
                    } else {
                        res >>= 1;
                    }
                    one >>= 2;
                }

                // SAFETY: the result is positive and fits in an integer with half as many bits.
                // Inform the optimizer about it.
                unsafe {
                    intrinsics::assume(0 < res);
                    intrinsics::assume(res < 1 << (Self::BITS / 2));
                }

                res
            }
        }
    };
}

unsigned_isqrt!(u128);
