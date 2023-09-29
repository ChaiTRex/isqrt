use core::intrinsics;

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
                // I would like to implement it as
                // ```
                // self.checked_isqrt().expect("argument of integer square root must be non-negative")
                // ```
                // but `expect` is not yet stable as a `const fn`.
                match self.checked_isqrt() {
                    Some(sqrt) => sqrt,
                    None => panic!("argument of integer square root must be non-negative"),
                }
            }
        }
    };
}

signed_isqrt!(i8, u8);
signed_isqrt!(i16, u16);
signed_isqrt!(i32, u32);
signed_isqrt!(i64, u64);
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

unsigned_isqrt!(u8);
unsigned_isqrt!(u16);
unsigned_isqrt!(u32);
unsigned_isqrt!(u64);
unsigned_isqrt!(u128);
