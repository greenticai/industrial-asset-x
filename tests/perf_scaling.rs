use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use industrial_asset_x::validate_catalog;

fn run_workload(threads: usize) -> Duration {
    let start = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|_| std::thread::spawn(|| validate_catalog(Path::new("."))))
        .collect();

    for handle in handles {
        let report = handle.join().expect("worker thread should complete");
        assert!(report.valid, "{:?}", report.diagnostics);
    }

    start.elapsed()
}

#[test]
fn validation_scaling_should_not_degrade_badly() {
    let single = run_workload(1);
    let four = run_workload(4);
    let allowed = single.mul_f64(12.0).max(Duration::from_millis(500));

    assert!(
        four <= allowed,
        "4 validation threads slower than expected: single={single:?}, four={four:?}, allowed={allowed:?}",
    );
}

#[test]
fn validation_completes_before_timeout() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let elapsed = run_workload(4);
        tx.send(elapsed).expect("timeout receiver should be open");
    });

    let elapsed = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("catalog validation should not hang");
    assert!(elapsed < Duration::from_secs(3));
}
