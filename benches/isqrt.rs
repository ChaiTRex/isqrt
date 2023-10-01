use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

#[allow(unused_mut)]
pub fn criterion_benchmark(c: &mut Criterion) {
    macro_rules! random_iter {
        ($type:ty) => {
            thread_rng().sample_iter::<$type, Uniform<$type>>(Uniform::new_inclusive(
                <$type>::MIN,
                <$type>::MAX,
            ))
        };
    }
    let mut random_i8s = random_iter!(i8);
    let mut random_u8s = random_iter!(u8);
    let mut random_i16s = random_iter!(i16);
    let mut random_u16s = random_iter!(u16);
    let mut random_i32s = random_iter!(i32);
    let mut random_u32s = random_iter!(u32);
    let mut random_i64s = random_iter!(i64);
    let mut random_u64s = random_iter!(u64);
    let mut random_i128s = random_iter!(i128);
    let mut random_u128s = random_iter!(u128);

    macro_rules! benches {
        ($c:ident, $signed_type:ty, $signed_randoms:ident, $unsigned_type:ty, $unsigned_randoms:ident) => {
            $c.bench_function(concat!("original_", stringify!($signed_type)), |b| {
                use isqrt::original::SignedIsqrt;

                b.iter(|| black_box(black_box($signed_randoms.next().unwrap()).checked_isqrt()))
            });

            $c.bench_function(concat!("floating_", stringify!($signed_type)), |b| {
                use isqrt::floating_point::SignedIsqrt;

                b.iter(|| black_box(black_box($signed_randoms.next().unwrap()).checked_isqrt()))
            });

            $c.bench_function(concat!("karatsuba_", stringify!($signed_type)), |b| {
                use isqrt::karatsuba::SignedIsqrt;

                b.iter(|| black_box(black_box($signed_randoms.next().unwrap()).checked_isqrt()))
            });

            $c.bench_function(concat!("original_", stringify!($unsigned_type)), |b| {
                use isqrt::original::UnsignedIsqrt;

                b.iter(|| black_box(black_box($unsigned_randoms.next().unwrap()).isqrt()))
            });

            $c.bench_function(concat!("floating_", stringify!($unsigned_type)), |b| {
                use isqrt::floating_point::UnsignedIsqrt;

                b.iter(|| black_box(black_box($unsigned_randoms.next().unwrap()).isqrt()))
            });

            $c.bench_function(concat!("karatsuba_", stringify!($unsigned_type)), |b| {
                use isqrt::karatsuba::UnsignedIsqrt;

                b.iter(|| black_box(black_box($unsigned_randoms.next().unwrap()).isqrt()))
            });
        };
    }

    benches!(c, i8, random_i8s, u8, random_u8s);
    benches!(c, i16, random_i16s, u16, random_u16s);
    benches!(c, i32, random_i32s, u32, random_u32s);
    benches!(c, i64, random_i64s, u64, random_u64s);
    benches!(c, i128, random_i128s, u128, random_u128s);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
