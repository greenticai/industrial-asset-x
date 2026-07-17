use criterion::{Criterion, criterion_group, criterion_main};
use industrial_asset_x::{catalog_summary, validate_catalog};
use std::path::Path;

fn bench_catalog_summary(c: &mut Criterion) {
    c.bench_function("catalog_summary", |b| {
        b.iter(|| {
            let summary = catalog_summary(Path::new("."));
            assert!(summary.contracts > 0);
        });
    });
}

fn bench_catalog_validation(c: &mut Criterion) {
    c.bench_function("catalog_validate", |b| {
        b.iter(|| {
            let report = validate_catalog(Path::new("."));
            assert!(report.valid, "{:?}", report.diagnostics);
        });
    });
}

criterion_group!(benches, bench_catalog_summary, bench_catalog_validation);
criterion_main!(benches);
