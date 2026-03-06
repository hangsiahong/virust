use criterion::{criterion_group, criterion_main, Criterion};
use virust_build::{SsgBuilder, SsgRoute};
use std::path::PathBuf;

fn bench_ssg_build_sequential(c: &mut Criterion) {
    c.bench_function("ssg_build_sequential_10_routes", |b| {
        b.iter(|| {
            let output = PathBuf::from("/tmp/bench_sequential");
            let _ = std::fs::remove_dir_all(&output);

            let mut builder = SsgBuilder::new(output.clone());
            builder.parallel_jobs = 1; // Sequential

            // Add 10 test routes
            for i in 0..10 {
                builder.routes.push(SsgRoute {
                    path: format!("/test{}", i),
                    handler: format!("handler{}", i),
                    revalidate: None,
                });
            }

            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let _ = builder.build().await;
                })
        })
    });
}

fn bench_ssg_build_parallel(c: &mut Criterion) {
    c.bench_function("ssg_build_parallel_10_routes", |b| {
        b.iter(|| {
            let output = PathBuf::from("/tmp/bench_parallel");
            let _ = std::fs::remove_dir_all(&output);

            let mut builder = SsgBuilder::new(output.clone());
            builder.parallel_jobs = 4; // Parallel

            // Add 10 test routes
            for i in 0..10 {
                builder.routes.push(SsgRoute {
                    path: format!("/test{}", i),
                    handler: format!("handler{}", i),
                    revalidate: None,
                });
            }

            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let _ = builder.build().await;
                })
        })
    });
}

criterion_group!(benches, bench_ssg_build_sequential, bench_ssg_build_parallel);
criterion_main!(benches);
