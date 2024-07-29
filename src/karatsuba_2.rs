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

    let mut n: usize = 0;
    let mut isqrt_n: usize = 0;
    while n < result.len() {
        result[n] = (isqrt_n as u8, (n - isqrt_n.pow(2)) as u8);

        n += 1;
        if n == (isqrt_n + 1).pow(2) {
            isqrt_n += 1;
        }
    }

    result
};

macro_rules! first_stage {
    ($original_bits:literal, $n:ident) => {{
        const N_SHIFT: u32 = $original_bits - 8;
        let n = $n >> N_SHIFT;

        ISQRT_AND_REMAINDER_8_BIT[n as usize]
    }};
}

macro_rules! middle_stage {
    ($original_bits:literal, $ty:ty, $n:ident, $s:ident, $r:ident) => {{
        const N_SHIFT: u32 = $original_bits - <$ty>::BITS;
        let n = ($n >> N_SHIFT) as $ty;

        const HALF_BITS: u32 = <$ty>::BITS >> 1;
        const QUARTER_BITS: u32 = <$ty>::BITS >> 2;
        const LOWER_HALF_1_BITS: $ty = (1 << HALF_BITS) - 1;
        const LOWEST_QUARTER_1_BITS: $ty = (1 << QUARTER_BITS) - 1;

        let lo = n & LOWER_HALF_1_BITS;
        let numerator = (($r as $ty) << QUARTER_BITS) | (lo >> QUARTER_BITS);
        let denominator = ($s as $ty) << 1;
        let q = numerator / denominator;
        let u = numerator % denominator;
        let mut s = ($s << QUARTER_BITS) as $ty + q;
        let (mut r, overflow) =
            ((u << QUARTER_BITS) | (lo & LOWEST_QUARTER_1_BITS)).overflowing_sub(q * q);
        if overflow {
            r = r.wrapping_add(2 * s - 1);
            s -= 1;
        }
        (s, r)
    }};
}

macro_rules! last_stage {
    ($ty:ty, $n:ident, $s:ident, $r:ident) => {{
        const HALF_BITS: u32 = <$ty>::BITS >> 1;
        const QUARTER_BITS: u32 = <$ty>::BITS >> 2;
        const LOWER_HALF_1_BITS: $ty = (1 << HALF_BITS) - 1;
        const LOWEST_QUARTER_1_BITS: $ty = (1 << QUARTER_BITS) - 1;

        let lo = $n & LOWER_HALF_1_BITS;
        let numerator = (($r as $ty) << QUARTER_BITS) | (lo >> QUARTER_BITS);
        let denominator = ($s as $ty) << 1;
        let q = numerator / denominator;
        let mut s = ($s << QUARTER_BITS) as $ty + q;
        let (s_squared, overflow) = s.overflowing_mul(s);
        if overflow || s_squared > $n {
            s -= 1;
        }
        s
    }};
}

const fn karatsuba_isqrt_8(n: u8) -> u8 {
    ISQRT_AND_REMAINDER_8_BIT[n as usize].0
}
const fn karatsuba_isqrt_16(mut n: u16) -> u16 {
    if n == 0 {
        return 0;
    }
    const EVEN_BITMASK: u32 = u32::MAX & !1;
    let precondition_shift = n.leading_zeros() & EVEN_BITMASK;
    n <<= precondition_shift;

    let (s, r) = first_stage!(16, n);
    let s = last_stage!(u16, n, s, r);

    let result_shift = precondition_shift >> 1;
    s >> result_shift
}
const fn karatsuba_isqrt_32(mut n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    const EVEN_BITMASK: u32 = u32::MAX & !1;
    let precondition_shift = n.leading_zeros() & EVEN_BITMASK;
    n <<= precondition_shift;

    let (s, r) = first_stage!(32, n);
    let (s, r) = middle_stage!(32, u16, n, s, r);
    let s = last_stage!(u32, n, s, r);

    let result_shift = precondition_shift >> 1;
    s >> result_shift
}
const fn karatsuba_isqrt_64(mut n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    const EVEN_BITMASK: u32 = u32::MAX & !1;
    let precondition_shift = n.leading_zeros() & EVEN_BITMASK;
    n <<= precondition_shift;

    let (s, r) = first_stage!(64, n);
    let (s, r) = middle_stage!(64, u16, n, s, r);
    let (s, r) = middle_stage!(64, u32, n, s, r);
    let s = last_stage!(u64, n, s, r);

    let result_shift = precondition_shift >> 1;
    s >> result_shift
}
const fn karatsuba_isqrt_128(mut n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    const EVEN_BITMASK: u32 = u32::MAX & !1;
    let precondition_shift = n.leading_zeros() & EVEN_BITMASK;
    n <<= precondition_shift;

    let (s, r) = first_stage!(128, n);
    let (s, r) = middle_stage!(128, u16, n, s, r);
    let (s, r) = middle_stage!(128, u32, n, s, r);
    let (s, r) = middle_stage!(128, u64, n, s, r);
    let s = last_stage!(u128, n, s, r);

    let result_shift = precondition_shift >> 1;
    s >> result_shift
}

impl SignedIsqrt for i8 {
    #[inline(always)]
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = karatsuba_isqrt_8(self as _) as Self;
            const ISQRT_MAX: i8 = karatsuba_isqrt_8(<i8>::MAX as _) as _;
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

impl UnsignedIsqrt for u8 {
    #[inline(always)]
    fn isqrt(self) -> Self {
        let result = karatsuba_isqrt_8(self);
        unsafe {
            intrinsics::assume(result < 1 << ((<u8>::BITS as Self) >> 1));
        }
        result
    }
}

impl SignedIsqrt for i16 {
    #[inline(always)]
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = karatsuba_isqrt_16(self as _) as Self;
            const ISQRT_MAX: i16 = karatsuba_isqrt_16(<i16>::MAX as _) as _;
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

impl UnsignedIsqrt for u16 {
    #[inline(always)]
    fn isqrt(self) -> Self {
        let result = karatsuba_isqrt_16(self);
        unsafe {
            intrinsics::assume(result < 1 << ((<u16>::BITS as Self) >> 1));
        }
        result
    }
}

impl SignedIsqrt for i32 {
    #[inline(always)]
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = karatsuba_isqrt_32(self as _) as Self;
            const ISQRT_MAX: i32 = karatsuba_isqrt_32(<i32>::MAX as _) as _;
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

impl UnsignedIsqrt for u32 {
    #[inline(always)]
    fn isqrt(self) -> Self {
        let result = karatsuba_isqrt_32(self);
        unsafe {
            intrinsics::assume(result < 1 << ((<u32>::BITS as Self) >> 1));
        }
        result
    }
}

impl SignedIsqrt for i64 {
    #[inline(always)]
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = karatsuba_isqrt_64(self as _) as Self;
            const ISQRT_MAX: i64 = karatsuba_isqrt_64(<i64>::MAX as _) as _;
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

impl UnsignedIsqrt for u64 {
    #[inline(always)]
    fn isqrt(self) -> Self {
        let result = karatsuba_isqrt_64(self);
        unsafe {
            intrinsics::assume(result < 1 << ((<u64>::BITS as Self) >> 1));
        }
        result
    }
}

impl SignedIsqrt for i128 {
    #[inline(always)]
    fn checked_isqrt(self) -> Option<Self> {
        (self >= 0).then(|| {
            let result = karatsuba_isqrt_128(self as _) as Self;
            const ISQRT_MAX: i128 = karatsuba_isqrt_128(<i128>::MAX as _) as _;
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

impl UnsignedIsqrt for u128 {
    #[inline(always)]
    fn isqrt(self) -> Self {
        let result = karatsuba_isqrt_128(self);
        unsafe {
            intrinsics::assume(result < 1 << ((<u128>::BITS as Self) >> 1));
        }
        result
    }
}
