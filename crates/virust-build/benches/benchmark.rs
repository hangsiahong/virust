//! # SSG Build Performance Benchmarks
//!
//! This benchmark compares sequential vs parallel SSG build performance to
//! validate the parallel rendering optimization implemented in Task 142.
//!
//! ## Methodology
//!
//! The benchmark tests SSG builds with 10 routes using different parallel job
//! configurations:
//! - **Sequential** (1 job): Routes are built one at a time
//! - **Parallel** (4 jobs): Routes are built concurrently with a semaphore
//!
//! The benchmark uses a pre-created tokio runtime to avoid runtime creation
//! overhead contaminating the measurements. Filesystem cleanup is handled
//! within each iteration.
//!
//! ## Expected Results
//!
//! With simple file operations (current implementation), parallel and sequential
//! performance may be similar due to:
//! - Low per-route overhead (simple file writes)
//! - Semaphore and task spawning overhead
//!
//! In production with complex component rendering, data fetching, and template
//! processing, the parallel version should show significant improvements,
//! especially with:
//! - More routes (50+)
//! - Slower per-route operations (database queries, API calls)
//! - Higher CPU core counts
//!
//! ## Running the Benchmark
//!
//! ```bash
//! cargo bench -p virust-build
//! ```
//!
//! Results are saved to `target/criterion/` with detailed statistics and graphs.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use virust_build::{SsgBuilder, SsgRoute};
use std::path::PathBuf;

fn bench_ssg_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("ssg_build");

    // Configure benchmark timing
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(5));

    // Create tokio runtime ONCE to avoid runtime creation overhead
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Benchmark both sequential and parallel configurations
    for (jobs, name) in [(1, "sequential"), (4, "parallel")] {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &jobs,
            |b, &jobs| {
                b.iter(|| {
                    // Use unique output directory per job configuration
                    let output = PathBuf::from(format!("/tmp/bench_ssg_{}", jobs));

                    // Clean up before each iteration
                    let _ = std::fs::remove_dir_all(&output);

                    // Create builder with configured parallelism
                    let mut builder = SsgBuilder::new(output.clone());
                    builder.parallel_jobs = jobs;

                    // Add 10 test routes
                    for i in 0..10 {
                        builder.routes.push(SsgRoute {
                            path: format!("/test{}", i),
                            handler: format!("handler{}", i),
                            revalidate: None,
                        });
                    }

                    // Run the build on the pre-created runtime
                    rt.block_on(async {
                        let _ = black_box(builder.build().await);
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_ssg_build);
criterion_main!(benches);
