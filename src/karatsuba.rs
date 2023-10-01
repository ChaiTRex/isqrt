use core::intrinsics;

pub trait SignedIsqrt: Sized {
    fn checked_isqrt(self) -> Option<Self>;
    fn isqrt(self) -> Self;
}
pub trait UnsignedIsqrt {
    fn isqrt(self) -> Self;
}

const ISQRT_AND_REMAINDER_8_BIT: [(u8, u8); 256] = {
    let mut result = [(0, 0); 256];

    let mut sqrt = 0;
    let mut i = 0;
    'outer: loop {
        let mut remaining = 2 * sqrt + 1;
        while remaining > 0 {
            result[i as usize] = (sqrt, 2 * sqrt + 1 - remaining);
            i += 1;
            if i >= result.len() {
                break 'outer;
            }
            remaining -= 1;
        }
        sqrt += 1;
    }

    result
};

const fn karatsuba_sqrt_8(n: u8) -> u8 {
    ISQRT_AND_REMAINDER_8_BIT[n as usize].0
}

const fn karatsuba_sqrt_with_remainder_8(n: u8) -> (u8, u8) {
    ISQRT_AND_REMAINDER_8_BIT[n as usize]
}

macro_rules! karatsuba_sqrt {
    ($FullBitsT:ty, $karatsuba_sqrt:ident, $karatsuba_sqrt_with_remainder:ident, $HalfBitsT:ty, $karatsuba_sqrt_half:ident, $karatsuba_sqrt_with_remainder_half:ident) => {
        const fn $karatsuba_sqrt(mut n: $FullBitsT) -> $FullBitsT {
            // Performs a Karatsuba square root.
            // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

            const HALF_BITS: u32 = <$FullBitsT>::BITS >> 1;
            const QUARTER_BITS: u32 = <$FullBitsT>::BITS >> 2;

            let leading_zeros = n.leading_zeros();
            let result = if leading_zeros >= HALF_BITS {
                $karatsuba_sqrt_half(n as $HalfBitsT) as $FullBitsT
            } else {
                // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
                let precondition_shift = leading_zeros & (HALF_BITS - 2);
                n <<= precondition_shift;

                let hi = (n >> HALF_BITS) as $HalfBitsT;
                let lo = n & (<$HalfBitsT>::MAX as $FullBitsT);

                let (s_prime, r_prime) = $karatsuba_sqrt_with_remainder_half(hi);

                let numerator = ((r_prime as $FullBitsT) << QUARTER_BITS) | (lo >> QUARTER_BITS);
                let denominator = (s_prime as $FullBitsT) << 1;

                let q = numerator / denominator;
                let u = numerator % denominator;

                let mut s = (s_prime << QUARTER_BITS) as $FullBitsT + q;
                if ((u << QUARTER_BITS) | (lo & ((1 << QUARTER_BITS) - 1))) < q * q {
                    s -= 1;
                }
                s >> (precondition_shift >> 1)
            };

            result
        }

        #[allow(dead_code)]
        const fn $karatsuba_sqrt_with_remainder(mut n: $FullBitsT) -> ($FullBitsT, $FullBitsT) {
            // Performs a Karatsuba square root.
            // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

            const HALF_BITS: u32 = <$FullBitsT>::BITS >> 1;
            const QUARTER_BITS: u32 = <$FullBitsT>::BITS >> 2;

            let leading_zeros = n.leading_zeros();
            let result = if leading_zeros >= HALF_BITS {
                let (s, r) = $karatsuba_sqrt_with_remainder_half(n as $HalfBitsT);
                (s as $FullBitsT, r as $FullBitsT)
            } else {
                // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
                let precondition_shift = leading_zeros & (HALF_BITS - 2);
                n <<= precondition_shift;

                let hi = (n >> HALF_BITS) as $HalfBitsT;
                let lo = n & (<$HalfBitsT>::MAX as $FullBitsT);

                let (s_prime, r_prime) = $karatsuba_sqrt_with_remainder_half(hi);

                let numerator = ((r_prime as $FullBitsT) << QUARTER_BITS) | (lo >> QUARTER_BITS);
                let denominator = (s_prime as $FullBitsT) << 1;

                let q = numerator / denominator;
                let u = numerator % denominator;

                let mut s = (s_prime << QUARTER_BITS) as $FullBitsT + q;
                let (mut r, overflow) =
                    ((u << QUARTER_BITS) | (lo & ((1 << QUARTER_BITS) - 1))).overflowing_sub(q * q);
                if overflow {
                    r = r.wrapping_add((s << 1) - 1);
                    s -= 1;
                }
                (
                    s >> (precondition_shift >> 1),
                    r >> (precondition_shift >> 1),
                )
            };

            result
        }
    };
}

karatsuba_sqrt!(
    u16,
    karatsuba_sqrt_16,
    karatsuba_sqrt_with_remainder_16,
    u8,
    karatsuba_sqrt_8,
    karatsuba_sqrt_with_remainder_8
);
karatsuba_sqrt!(
    u32,
    karatsuba_sqrt_32,
    karatsuba_sqrt_with_remainder_32,
    u16,
    karatsuba_sqrt_16,
    karatsuba_sqrt_with_remainder_16
);
karatsuba_sqrt!(
    u64,
    karatsuba_sqrt_64,
    karatsuba_sqrt_with_remainder_64,
    u32,
    karatsuba_sqrt_32,
    karatsuba_sqrt_with_remainder_32
);
karatsuba_sqrt!(
    u128,
    karatsuba_sqrt_128,
    karatsuba_sqrt_with_remainder_128,
    u64,
    karatsuba_sqrt_64,
    karatsuba_sqrt_with_remainder_64
);

macro_rules! isqrt_impl {
    ($signed_type:ty, $unsigned_type:ty, $karatsuba_sqrt:ident) => {
        impl SignedIsqrt for $signed_type {
            #[inline(always)]
            fn checked_isqrt(self) -> Option<Self> {
                (self >= 0).then(|| {
                    let result = $karatsuba_sqrt(self as _) as Self;

                    // SAFETY: the result is nonnegative and less than or equal to `i16::MAX.isqrt()`.
                    // Inform the optimizer about it.
                    const ISQRT_MAX: $signed_type = $karatsuba_sqrt(<$signed_type>::MAX as _) as _;
                    unsafe {
                        intrinsics::assume(0 <= result);
                        intrinsics::assume(result <= ISQRT_MAX);
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

        impl UnsignedIsqrt for $unsigned_type {
            #[inline(always)]
            fn isqrt(self) -> Self {
                let result = $karatsuba_sqrt(self);

                // SAFETY: the result fits in an integer with half as many bits.
                // Inform the optimizer about it.
                unsafe {
                    intrinsics::assume(result < 1 << ((<$unsigned_type>::BITS as Self) >> 1));
                }

                result
            }
        }
    };
}

isqrt_impl!(i8, u8, karatsuba_sqrt_8);
isqrt_impl!(i16, u16, karatsuba_sqrt_16);
isqrt_impl!(i32, u32, karatsuba_sqrt_32);
isqrt_impl!(i64, u64, karatsuba_sqrt_64);
isqrt_impl!(i128, u128, karatsuba_sqrt_128);
