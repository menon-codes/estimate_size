use criterion::{Criterion, criterion_group, criterion_main};
use estimate_size::EstimateSize;
use std::hint::black_box;

/// Benchmark the most dramatic case: collecting filtered data into Vec
/// This shows the biggest performance difference because Vec reallocations are expensive
fn bench_vec_reallocation_torture_test(c: &mut Criterion) {
    c.bench_function("vec_torture_without_estimate", |b| {
        b.iter(|| {
            // Worst case: we don't know the final size, Vec starts small and reallocates many times
            let iter = (0..100_000).filter(|x| x % 997 == 0); // Very sparse filter, ~100 elements
            let vec: Vec<i32> = black_box(iter.collect());
            vec.len()
        })
    });

    c.bench_function("vec_torture_with_estimate", |b| {
        b.iter(|| {
            // Best case: we provide a good size estimate, Vec allocates correctly from the start
            let iter = (0..100_000)
                .filter(|x| x % 997 == 0)
                .estimate_exact_size(100); // We know there will be ~100 elements
            let vec: Vec<i32> = black_box(iter.collect());
            vec.len()
        })
    });
}

/// Show the benefit for large memory allocations
fn bench_large_allocation(c: &mut Criterion) {
    c.bench_function("large_structs_without_estimate", |b| {
        b.iter(|| {
            let iter = (0..50_000)
                .filter(|x| x % 10 == 0) // 1/10th remain
                .map(|i| vec![i; 100]); // Each element is a large Vec

            let result: Vec<Vec<i32>> = black_box(iter.collect());
            result.len()
        })
    });

    c.bench_function("large_structs_with_estimate", |b| {
        b.iter(|| {
            let iter = (0..50_000)
                .filter(|x| x % 10 == 0)
                .map(|i| vec![i; 100])
                .estimate_exact_size(5_000); // We know exactly: 50_000 / 10 = 5_000

            let result: Vec<Vec<i32>> = black_box(iter.collect());
            result.len()
        })
    });
}

criterion_group!(
    benches,
    bench_vec_reallocation_torture_test,
    bench_large_allocation
);
criterion_main!(benches);
