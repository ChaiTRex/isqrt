use core::intrinsics;

pub trait SignedIsqrt: Sized {
    fn checked_isqrt(self) -> Option<Self>;
    fn isqrt(self) -> Self;
}
pub trait UnsignedIsqrt {
    fn isqrt(self) -> Self;
}

macro_rules! sqrt_impls {
    ($signed_type:ty, $unsigned_type:ty, $const_isqrt:ident, $fast_isqrt:ident, $combined_isqrt:ident) => {
        #[inline(always)]
        const fn $combined_isqrt(n: $unsigned_type) -> $unsigned_type {
            // SAFETY: identical inputs to both functions give identical results.
            unsafe { intrinsics::const_eval_select((n,), $const_isqrt, $fast_isqrt) }
        }

        impl SignedIsqrt for $signed_type {
            #[inline]
            fn checked_isqrt(self) -> Option<Self> {
                if self < 0 {
                    None
                } else {
                    let result: Self = $combined_isqrt(self as $unsigned_type) as Self;

                    // Make sure to use this `const` rather than just calculating it in `assume` below. Doing so
                    // ensures that the calculation is done at compile-time rather than during every single `isqrt`
                    // call.
                    const MAX_RESULT: $signed_type =
                        $const_isqrt(<$signed_type>::MAX as $unsigned_type) as $signed_type;
                    // SAFETY: the result is nonnegative and less than or equal to `i8::MAX.isqrt()`.
                    // Inform the optimizer about it.
                    unsafe {
                        intrinsics::assume(0 <= result);
                        intrinsics::assume(result <= MAX_RESULT);
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

        impl UnsignedIsqrt for $unsigned_type {
            #[inline(always)]
            fn isqrt(self) -> Self {
                let result = $combined_isqrt(self);

                // Make sure to use this `const` rather than just calculating it in `assume` below. Doing so ensures
                // that the calculation is done at compile-time rather than during every single `isqrt` call.
                const MAX_RESULT: $unsigned_type = $const_isqrt(<$unsigned_type>::MAX);
                // SAFETY: The square root cannot exceed the square root of the maximum input.
                // Inform the optimizer.
                unsafe {
                    intrinsics::assume(result <= MAX_RESULT);
                }

                result
            }
        }
    };
}

sqrt_impls!(
    i8,
    u8,
    karatsuba_isqrt_8,
    karatsuba_isqrt_8,
    combined_isqrt_8
);
sqrt_impls!(
    i16,
    u16,
    karatsuba_isqrt_16,
    floating_isqrt_16,
    combined_isqrt_16
);
sqrt_impls!(
    i32,
    u32,
    karatsuba_isqrt_32,
    floating_isqrt_32,
    combined_isqrt_32
);
sqrt_impls!(
    i64,
    u64,
    karatsuba_isqrt_64,
    floating_isqrt_64,
    combined_isqrt_64
);
sqrt_impls!(
    i128,
    u128,
    karatsuba_isqrt_128,
    floating_isqrt_128,
    combined_isqrt_128
);

/*** KARATSUBA METHOD ***/

const ISQRT_8_BIT: [u8; 256] = {
    let mut result = [0; 256];

    let mut sqrt = 0;
    let mut i = 0;
    'outer: loop {
        let mut remaining = 2 * sqrt + 1;
        while remaining > 0 {
            result[i as usize] = sqrt;
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

// The first three bits of each entry are the last three bits of the square root. The next five bits are the remainder.
const ISQRT_AND_REMAINDER_8_BIT: [u8; 256] = {
    let mut result = [0; 256];

    let mut sqrt = 0;
    let mut i = 0;
    'outer: loop {
        let mut remaining = 2 * sqrt + 1;
        while remaining > 0 {
            result[i as usize] = (sqrt << 5) | (2 * sqrt + 1 - remaining);
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

#[inline(always)]
const fn karatsuba_isqrt_8(n: u8) -> u8 {
    ISQRT_8_BIT[n as usize]
}

#[inline(always)]
const fn karatsuba_isqrt_with_remainder_8(n: u8) -> (u8, u8) {
    let table_entry = ISQRT_AND_REMAINDER_8_BIT[n as usize];
    let s = ((n >= 64) as u8) << 3 | (table_entry >> 5);
    let r = table_entry & 0b11111;
    (s, r)
}

macro_rules! karatsuba_isqrt {
    ($FullBitsT:ty, $karatsuba_isqrt:ident, $karatsuba_isqrt_with_remainder:ident, $HalfBitsT:ty, $karatsuba_isqrt_half:ident, $karatsuba_isqrt_with_remainder_half:ident) => {
        const fn $karatsuba_isqrt(mut n: $FullBitsT) -> $FullBitsT {
            // Performs a Karatsuba square root.
            // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

            const HALF_BITS: u32 = <$FullBitsT>::BITS >> 1;
            const QUARTER_BITS: u32 = <$FullBitsT>::BITS >> 2;

            let leading_zeros = n.leading_zeros();
            let result = if leading_zeros >= HALF_BITS {
                $karatsuba_isqrt_half(n as $HalfBitsT) as $FullBitsT
            } else {
                // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
                let precondition_shift = leading_zeros & (HALF_BITS - 2);
                n <<= precondition_shift;

                let hi = (n >> HALF_BITS) as $HalfBitsT;
                let lo = n & (<$HalfBitsT>::MAX as $FullBitsT);

                let (s_prime, r_prime) = $karatsuba_isqrt_with_remainder_half(hi);

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
        const fn $karatsuba_isqrt_with_remainder(mut n: $FullBitsT) -> ($FullBitsT, $FullBitsT) {
            // Performs a Karatsuba square root.
            // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

            const HALF_BITS: u32 = <$FullBitsT>::BITS >> 1;
            const QUARTER_BITS: u32 = <$FullBitsT>::BITS >> 2;

            let leading_zeros = n.leading_zeros();
            let result = if leading_zeros >= HALF_BITS {
                let (s, r) = $karatsuba_isqrt_with_remainder_half(n as $HalfBitsT);
                (s as $FullBitsT, r as $FullBitsT)
            } else {
                // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
                let precondition_shift = leading_zeros & (HALF_BITS - 2);
                n <<= precondition_shift;

                let hi = (n >> HALF_BITS) as $HalfBitsT;
                let lo = n & (<$HalfBitsT>::MAX as $FullBitsT);

                let (s_prime, r_prime) = $karatsuba_isqrt_with_remainder_half(hi);

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

karatsuba_isqrt!(
    u16,
    karatsuba_isqrt_16,
    karatsuba_isqrt_with_remainder_16,
    u8,
    karatsuba_isqrt_8,
    karatsuba_isqrt_with_remainder_8
);
karatsuba_isqrt!(
    u32,
    karatsuba_isqrt_32,
    karatsuba_isqrt_with_remainder_32,
    u16,
    karatsuba_isqrt_16,
    karatsuba_isqrt_with_remainder_16
);
karatsuba_isqrt!(
    u64,
    karatsuba_isqrt_64,
    karatsuba_isqrt_with_remainder_64,
    u32,
    karatsuba_isqrt_32,
    karatsuba_isqrt_with_remainder_32
);
karatsuba_isqrt!(
    u128,
    karatsuba_isqrt_128,
    karatsuba_isqrt_with_remainder_128,
    u64,
    karatsuba_isqrt_64,
    karatsuba_isqrt_with_remainder_64
);

/*** FLOATING POINT METHOD ***/

#[allow(dead_code)]
#[inline]
fn floating_isqrt_8(n: u8) -> u8 {
    (n as f32).sqrt() as u8
}

#[inline]
fn floating_isqrt_16(n: u16) -> u16 {
    (n as f32).sqrt() as u16
}

#[inline]
fn floating_isqrt_32(n: u32) -> u32 {
    (n as f64).sqrt() as u32
}

fn floating_isqrt_64(n: u64) -> u64 {
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
    // near `u64::MAX`, where the ranges contain loosely 2^(64 - 53) = 2^11 inputs. The smallest distance between
    // perfect squares after 2^53 is loosely (2^26 + 1)^2 - (2^26)^2 = 2^27 - 1. Since the smallest distance between
    // perfect squares after inaccuracies appear is always much larger than the size of the largest range of
    // inputs, there cannot be more than one perfect square in a range of inputs.
    //
    // Thus, the correct output will be the floor of the representative's square root plus -1, 0, or 1.

    // Avoid overflows when getting the result squared or the result plus one squared.
    if n < ((1 << 32) - 2) * ((1 << 32) - 2) {
        let result = (n as f64).sqrt();
        // SAFETY: Guaranteed to not be a NaN or an infinity and to, except for the fractional part, be in `u64`
        // range.
        let result = unsafe { result.to_int_unchecked::<u64>() };
        let result_squared = result * result;
        if n < result_squared {
            result - 1
        } else if n < result_squared + (result << 1) + 1 {
            result
        } else {
            result + 1
        }
    } else if n < ((1 << 32) - 1) * ((1 << 32) - 1) {
        (1 << 32) - 2
    } else {
        (1 << 32) - 1
    }
}

fn floating_isqrt_128(mut n: u128) -> u128 {
    // Performs a Karatsuba square root.
    // https://web.archive.org/web/20230511212802/https://inria.hal.science/inria-00072854v1/file/RR-3805.pdf

    type HalfBitsT = u64;

    const HALF_BITS: u32 = HalfBitsT::BITS;
    const QUARTER_BITS: u32 = HalfBitsT::BITS >> 1;

    let leading_zeros = n.leading_zeros();
    if leading_zeros >= HALF_BITS {
        floating_isqrt_64(n as HalfBitsT) as u128
    } else {
        // Either the most-significant bit or its neighbor must be a one, so we shift left to make that happen.
        let precondition_shift = leading_zeros & (HALF_BITS - 2);
        n <<= precondition_shift;

        let hi = (n >> HALF_BITS) as HalfBitsT;
        let lo = n & (HalfBitsT::MAX as u128);

        let s_prime = hi.isqrt();
        let r_prime = hi - s_prime * s_prime;

        let numerator = ((r_prime as u128) << QUARTER_BITS) | (lo >> QUARTER_BITS);
        let denominator = (s_prime as u128) << 1;

        let q = numerator / denominator;
        let u = numerator % denominator;

        let mut s = (s_prime << QUARTER_BITS) as u128 + q;
        if (u << QUARTER_BITS) | (lo & ((1 << QUARTER_BITS) - 1)) < q * q {
            s -= 1;
        }
        s >> (precondition_shift >> 1)
    }
}
