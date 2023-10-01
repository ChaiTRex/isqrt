macro_rules! tests {
    ($module:ident ; $($SignedT:ident $UnsignedT:ident),+) => {
        mod $module {
            $(
                mod $SignedT {
                    #[allow(unused)]
                    use crate::$module::SignedIsqrt;

                    fn isqrt_consistency_check(n: $SignedT) {
                        // `$SignedT::MIN` will be negative, so we don't want to handle `n` as if it's nonnegative.
                        if n >= 0 {
                            assert_eq!(
                                Some(n.isqrt()),
                                n.checked_isqrt(),
                                "`{n}.checked_isqrt()` should match `Some({n}.isqrt())`.",
                            );
                        }

                        let negative_n = n.wrapping_neg();
                        // `n` could be zero, so we don't want to handle `negative_n` as if it's negative.
                        if negative_n < 0 {
                            assert_eq!(
                                negative_n.checked_isqrt(),
                                None,
                                "`({negative_n}).checked_isqrt()` should be `None`, as {negative_n} is negative.",
                            );

                            std::panic::catch_unwind(|| (-n).isqrt()).expect_err(
                                &format!("`({negative_n}).isqrt()` should have panicked, as {negative_n} is negative.")
                            );
                        }
                    }

                    #[test]
                    fn test_isqrt() {
                        // Check the minimum value.
                        isqrt_consistency_check($SignedT::MIN);

                        // Check the square roots of the first and last 128 nonnegative values, of the powers of two minus one,
                        // and of the powers of two.
                        for n in (0..=127)
                            .chain($SignedT::MAX - 127..=$SignedT::MAX)
                            .chain((0..$SignedT::BITS - 1).map(|exponent| (1 << exponent) - 1))
                            .chain((0..$SignedT::BITS - 1).map(|exponent| 1 << exponent))
                        {
                            isqrt_consistency_check(n);
                            let sqrt_n = n.isqrt();

                            assert!(
                                sqrt_n * sqrt_n <= n,
                                "The integer square root of {n} should be lower than {sqrt_n} (the current return value of `{n}.isqrt()`)."
                            );

                            assert!(
                                (sqrt_n + 1).checked_mul(sqrt_n + 1).map(|higher_than_n| n < higher_than_n).unwrap_or(true),
                                "The integer square root of {n} should be higher than {sqrt_n} (the current return value of `{n}.isqrt()`)."
                            );
                        }
                    }

                    #[test]
                    #[cfg(not(miri))]
                    fn test_isqrt_extended() {
                        // Check the square roots of the first 1,024 perfect squares, halfway between perfect squares, and
                        // perfect squares minus one.
                        //
                        // This works because the nth perfect square is the sum of the first n odd numbers:
                        //
                        // 0 = 0
                        // 1 = 1
                        // 4 = 1 + 3
                        // 9 = 1 + 3 + 5
                        // 16 = 1 + 3 + 5 + 7
                        //
                        // Note also that the last odd number added in is the 2 times the square root of the previous perfect
                        // square plus one:
                        //
                        // 1 = 2*0 + 1
                        // 3 = 2*1 + 1
                        // 5 = 2*2 + 1
                        // 7 = 2*3 + 1
                        //
                        // That means we can add the square root of this perfect square once to get about halfway to the next
                        // perfect square, then we can add the square root of this perfect square again to get to the next
                        // perfect square minus one, then we can add one to get to the next perfect square.
                        //
                        // This allows us to test that the square roots of the current perfect square, halfway to the next
                        // perfect square, and the next perfect square minus one are all matching and all correct.
                        let mut n: $SignedT = 0;
                        for sqrt_n in 0..1024.min((1_u128 << (($SignedT::BITS - 1)/2)) - 1) as $SignedT {
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`{sqrt_n}.pow(2).isqrt()` should be {sqrt_n}."
                            );
                            isqrt_consistency_check(n);

                            n += sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`isqrt` of a number halfway between `{sqrt_n}.pow(2)` and `{}.pow(2)` should be {sqrt_n}.",
                                sqrt_n + 1
                            );
                            isqrt_consistency_check(n);

                            n += sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`({}.pow(2) - 1).isqrt()` should be {sqrt_n}.",
                                sqrt_n + 1
                            );
                            isqrt_consistency_check(n);

                            n += 1;
                        }

                        // Similarly, check the last 1,024 perfect squares.
                        let maximum_sqrt = $SignedT::MAX.isqrt(); // Maximum `isqrt` return value verified above.
                        let mut n = maximum_sqrt * maximum_sqrt;
                        assert_eq!(n.isqrt(), maximum_sqrt);
                        for sqrt_n in (maximum_sqrt - 1024.min((1_u128 << (($SignedT::BITS - 1)/2)) - 1) as $SignedT..maximum_sqrt).rev() {
                            n -= 1;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`({}.pow(2) - 1).isqrt()` should be {sqrt_n}.",
                                sqrt_n + 1
                            );
                            isqrt_consistency_check(n);

                            n -= sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`isqrt` of a number halfway between `{sqrt_n}.pow(2)` and `{}.pow(2)` should be {sqrt_n}.",
                                sqrt_n + 1
                            );
                            isqrt_consistency_check(n);

                            n -= sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`{sqrt_n}.pow(2).isqrt()` should be {sqrt_n}."
                            );
                            isqrt_consistency_check(n);
                        }
                    }
                }

                mod $UnsignedT {
                    #[allow(unused)]
                    use crate::$module::UnsignedIsqrt;

                    #[test]
                    fn test_isqrt() {
                        // Check the square roots of the first and last 128 nonnegative values, of the powers of two minus one,
                        // and of the powers of two.
                        for n in (0..=127)
                            .chain($UnsignedT::MAX - 127..=$UnsignedT::MAX)
                            .chain((0..$UnsignedT::BITS).map(|exponent| (1 << exponent) - 1))
                            .chain((0..$UnsignedT::BITS).map(|exponent| 1 << exponent))
                        {
                            let sqrt_n = n.isqrt();

                            assert!(
                                sqrt_n * sqrt_n <= n,
                                "The integer square root of {n} should be lower than {sqrt_n} (the current return value of `{n}.isqrt()`)."
                            );

                            assert!(
                                (sqrt_n + 1).checked_mul(sqrt_n + 1).map(|higher_than_n| n < higher_than_n).unwrap_or(true),
                                "The integer square root of {n} should be higher than {sqrt_n} (the current return value of `{n}.isqrt()`)."
                            );
                        }
                    }

                    #[test]
                    #[cfg(not(miri))]
                    fn test_isqrt_extended() {
                        // Check the square roots of the first 1,024 perfect squares, halfway between perfect squares, and
                        // perfect squares minus one.
                        //
                        // This works because the nth perfect square is the sum of the first n odd numbers:
                        //
                        // 0 = 0
                        // 1 = 1
                        // 4 = 1 + 3
                        // 9 = 1 + 3 + 5
                        // 16 = 1 + 3 + 5 + 7
                        //
                        // Note also that the last odd number added in is the 2 times the square root of the previous perfect
                        // square plus one:
                        //
                        // 1 = 2*0 + 1
                        // 3 = 2*1 + 1
                        // 5 = 2*2 + 1
                        // 7 = 2*3 + 1
                        //
                        // That means we can add the square root of this perfect square once to get about halfway to the next
                        // perfect square, then we can add the square root of this perfect square again to get to the next
                        // perfect square minus one, then we can add one to get to the next perfect square.
                        //
                        // This allows us to test that the square roots of the current perfect square, halfway to the next
                        // perfect square, and the next perfect square minus one are all matching and all correct.
                        let mut n: $UnsignedT = 0;
                        for sqrt_n in 0..1024.min((1_u128 << ($UnsignedT::BITS/2)) - 1) as $UnsignedT {
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`{sqrt_n}.pow(2).isqrt()` should be {sqrt_n}."
                            );

                            n += sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`isqrt` of a number halfway between `{sqrt_n}.pow(2)` and `{}.pow(2)` should be {sqrt_n}.",
                                sqrt_n + 1
                            );

                            n += sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`({}.pow(2) - 1).isqrt()` should be {sqrt_n}.",
                                sqrt_n + 1
                            );

                            n += 1;
                        }

                        // Similarly, check the last 1,024 perfect squares.
                        let maximum_sqrt = $UnsignedT::MAX.isqrt(); // Maximum `isqrt` return value verified above.
                        let mut n = maximum_sqrt * maximum_sqrt;
                        assert_eq!(n.isqrt(), maximum_sqrt);
                        for sqrt_n in (maximum_sqrt - 1024.min((1_u128 << ($UnsignedT::BITS/2)) - 1) as $UnsignedT..maximum_sqrt).rev() {
                            n -= 1;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`({}.pow(2) - 1).isqrt()` should be {sqrt_n}.",
                                sqrt_n + 1
                            );

                            n -= sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`isqrt` of a number halfway between `{sqrt_n}.pow(2)` and `{}.pow(2)` should be {sqrt_n}.",
                                sqrt_n + 1
                            );

                            n -= sqrt_n;
                            assert_eq!(
                                n.isqrt(),
                                sqrt_n,
                                "`{sqrt_n}.pow(2).isqrt()` should be {sqrt_n}."
                            );
                        }
                    }
                }
            )*
        }
    };
}

tests!(floating_point; i8 u8, i16 u16, i32 u32, i64 u64, i128 u128);
tests!(karatsuba; i8 u8, i16 u16, i32 u32, i64 u64, i128 u128);
tests!(original; i8 u8, i16 u16, i32 u32, i64 u64, i128 u128);
