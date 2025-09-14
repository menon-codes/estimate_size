use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use estimate_size::EstimateSize;
use std::collections::HashMap;
use std::hint::black_box;

/// Benchmark Vec::collect() performance with and without size estimates
fn bench_vec_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_collect");

    for size in [100, 1000, 10000, 100000].iter() {
        // Without estimate_size - Vec has to reallocate multiple times
        group.bench_with_input(
            BenchmarkId::new("without_estimate", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let iter = (0..size).filter(|x| x % 3 == 0); // Unknown size hint
                    let vec: Vec<i32> = black_box(iter.collect());
                    vec.len()
                })
            },
        );

        // With estimate_size - Vec can pre-allocate efficiently
        group.bench_with_input(BenchmarkId::new("with_estimate", size), size, |b, &size| {
            b.iter(|| {
                let iter = (0..size)
                    .filter(|x| x % 3 == 0)
                    .estimate_size(size as usize / 3, Some(size as usize / 2)); // Good estimate for filtered data
                let vec: Vec<i32> = black_box(iter.collect());
                vec.len()
            })
        });
    }
    group.finish();
}

/// Benchmark HashMap::from_iter() with capacity hints
fn bench_hashmap_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_collect");

    for size in [100, 1000, 10000].iter() {
        // Without size estimate - HashMap rehashes multiple times
        group.bench_with_input(
            BenchmarkId::new("without_estimate", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let iter = (0..size).filter(|x| x % 2 == 0).map(|x| (x, x * 2));
                    let map: HashMap<i32, i32> = black_box(iter.collect());
                    map.len()
                })
            },
        );

        // With size estimate - HashMap can reserve capacity upfront
        group.bench_with_input(BenchmarkId::new("with_estimate", size), size, |b, &size| {
            b.iter(|| {
                let iter = (0..size)
                    .filter(|x| x % 2 == 0)
                    .map(|x| (x, x * 2))
                    .estimate_exact_size(size as usize / 2); // We know exactly how many even numbers
                let map: HashMap<i32, i32> = black_box(iter.collect());
                map.len()
            })
        });
    }
    group.finish();
}

/// Benchmark String collection from chars
fn bench_string_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_collect");

    for size in [1000, 10000, 100000].iter() {
        // Without estimate - String reallocates as it grows
        group.bench_with_input(
            BenchmarkId::new("without_estimate", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let iter = (0..size).map(|i| char::from((65 + (i % 26)) as u8)); // A-Z chars
                    let filtered = iter.filter(|c| *c != 'E'); // Remove 'E' chars
                    let string: String = black_box(filtered.collect());
                    string.len()
                })
            },
        );

        // With estimate - String can reserve capacity
        group.bench_with_input(BenchmarkId::new("with_estimate", size), size, |b, &size| {
            b.iter(|| {
                let iter = (0..size).map(|i| char::from((65 + (i % 26)) as u8)); // A-Z chars
                let filtered = iter
                    .filter(|c| *c != 'E')
                    .estimate_size(size * 25 / 26, Some(size * 25 / 26)); // 25/26 chars remain
                let string: String = black_box(filtered.collect());
                string.len()
            })
        });
    }
    group.finish();
}

/// Benchmark nested Vec<Vec<T>> collection
fn bench_nested_vec_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_vec_collect");

    for chunks in [10, 50, 100].iter() {
        let chunk_size = 100;

        // Without estimate - each inner Vec reallocates
        group.bench_with_input(
            BenchmarkId::new("without_estimate", chunks),
            chunks,
            |b, &chunks| {
                b.iter(|| {
                    let iter = (0..chunks).map(|chunk_id| {
                        (chunk_id * chunk_size..(chunk_id + 1) * chunk_size)
                            .filter(|x| x % 7 != 0) // Filter out multiples of 7
                            .collect::<Vec<_>>()
                    });
                    let nested: Vec<Vec<i32>> = black_box(iter.collect());
                    nested.len()
                })
            },
        );

        // With estimate - inner Vecs can pre-allocate
        group.bench_with_input(
            BenchmarkId::new("with_estimate", chunks),
            chunks,
            |b, &chunks| {
                b.iter(|| {
                    let iter = (0..chunks).map(|chunk_id| {
                        (chunk_id * chunk_size..(chunk_id + 1) * chunk_size)
                            .filter(|x| x % 7 != 0)
                            .estimate_size(
                                chunk_size as usize * 6 / 7,
                                Some(chunk_size as usize * 6 / 7),
                            )
                            .collect::<Vec<_>>()
                    });
                    let nested: Vec<Vec<i32>> =
                        black_box(iter.estimate_exact_size(chunks as usize).collect());
                    nested.len()
                })
            },
        );
    }
    group.finish();
}

/// Benchmark iterator chains with size estimates
fn bench_iterator_chains(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterator_chains");

    for size in [1000, 5000, 10000].iter() {
        // Chain multiple iterators without size hints
        group.bench_with_input(
            BenchmarkId::new("without_estimate", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let iter1 = (0..size / 3).filter(|x| x % 2 == 0);
                    let iter2 = (size / 3..2 * size / 3).filter(|x| x % 3 == 0);
                    let iter3 = (2 * size / 3..size).filter(|x| x % 5 == 0);

                    let chained = iter1.chain(iter2).chain(iter3);
                    let vec: Vec<i32> = black_box(chained.collect());
                    vec.len()
                })
            },
        );

        // Chain with good size estimates
        group.bench_with_input(BenchmarkId::new("with_estimate", size), size, |b, &size| {
            b.iter(|| {
                let iter1 = (0..size / 3)
                    .filter(|x| x % 2 == 0)
                    .estimate_size(size as usize / 6, Some(size as usize / 6));
                let iter2 = (size / 3..2 * size / 3)
                    .filter(|x| x % 3 == 0)
                    .estimate_size(size as usize / 9, Some(size as usize / 9));
                let iter3 = (2 * size / 3..size)
                    .filter(|x| x % 5 == 0)
                    .estimate_size(size as usize / 15, Some(size as usize / 15));

                let chained = iter1.chain(iter2).chain(iter3);
                let vec: Vec<i32> = black_box(chained.collect());
                vec.len()
            })
        });
    }
    group.finish();
}

/// Benchmark large data processing pipeline
fn bench_data_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_pipeline");

    for size in [10000, 50000].iter() {
        // Complex pipeline without size estimates
        group.bench_with_input(
            BenchmarkId::new("without_estimate", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let processed: Vec<_> = (0..size)
                        .filter(|x| x % 2 == 0) // Keep evens
                        .map(|x| x * x) // Square them
                        .filter(|x| x % 100 < 50) // Keep lower half of hundreds
                        .map(|x| (x, x.to_string())) // Create tuples
                        .collect();

                    black_box(processed.len())
                })
            },
        );

        // Pipeline with size estimates at each step
        group.bench_with_input(BenchmarkId::new("with_estimate", size), size, |b, &size| {
            b.iter(|| {
                let processed: Vec<_> = (0..size)
                    .filter(|x| x % 2 == 0)
                    .estimate_size(size as usize / 2, Some(size as usize / 2)) // Half remain after even filter
                    .map(|x| x * x)
                    .filter(|x| x % 100 < 50)
                    .estimate_size(size as usize / 4, Some(size as usize / 4)) // Roughly quarter remain
                    .map(|x| (x, x.to_string()))
                    .collect();

                black_box(processed.len())
            })
        });
    }
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test scenarios where memory allocation is the primary bottleneck
    for size in [5000, 25000].iter() {
        // Large struct collections without size hints
        group.bench_with_input(
            BenchmarkId::new("large_structs_without", size),
            size,
            |b, &size| {
                #[derive(Clone)]
                #[allow(dead_code)]
                struct LargeStruct {
                    data: [u64; 16], // 128 bytes
                    id: usize,
                }

                b.iter(|| {
                    let iter = (0..size)
                        .filter(|x| x % 3 != 0) // Remove multiples of 3
                        .map(|id| LargeStruct {
                            data: [id as u64; 16],
                            id,
                        });

                    let vec: Vec<LargeStruct> = black_box(iter.collect());
                    vec.len()
                })
            },
        );

        // Large struct collections with size hints
        group.bench_with_input(
            BenchmarkId::new("large_structs_with", size),
            size,
            |b, &size| {
                #[derive(Clone)]
                #[allow(dead_code)]
                struct LargeStruct {
                    data: [u64; 16], // 128 bytes
                    id: usize,
                }

                b.iter(|| {
                    let iter = (0..size)
                        .filter(|x| x % 3 != 0)
                        .map(|id| LargeStruct {
                            data: [id as u64; 16],
                            id,
                        })
                        .estimate_size(size as usize * 2 / 3, Some(size as usize * 2 / 3)); // 2/3 remain

                    let vec: Vec<LargeStruct> = black_box(iter.collect());
                    vec.len()
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_vec_collect,
    bench_hashmap_collect,
    bench_string_collect,
    bench_nested_vec_collect,
    bench_iterator_chains,
    bench_data_pipeline,
    bench_memory_patterns
);
criterion_main!(benches);
