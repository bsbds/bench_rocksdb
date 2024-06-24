use std::sync::Arc;

use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rocksdb::{WriteBatch, DB};
use tokio::runtime::{Builder, Runtime};

const BENCH_PATH: &'static str = "rocksdb_bench";
const NUM_THREAD: usize = 2;
const KEY_SIZE: usize = 256;
const VALUE_SIZE: usize = 1024;
const NUM_KEYS: usize = 100;

fn build_runtime() -> Runtime {
    let rt = Builder::new_multi_thread()
        .worker_threads(NUM_THREAD)
        .enable_all()
        .build()
        .unwrap();

    rt
}

fn db_write(db: &DB) {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let value = vec![0; VALUE_SIZE];
    let batch = std::iter::repeat_with(|| {
        let mut arr = [0u8; KEY_SIZE];
        rng.fill(&mut arr[..]);
        arr
    })
    .take(NUM_KEYS)
    .fold(WriteBatch::default(), |mut b, k| {
        b.put(&k, value.clone());
        b
    });
    db.write(batch).unwrap();
}

fn clean_up() {
    std::fs::remove_dir_all(BENCH_PATH).unwrap();
}

fn rocksdb_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("rocksdb_benchmarks");
    let rt = build_runtime();
    let db = Arc::new(DB::open_default(BENCH_PATH).unwrap());

    group.bench_function("rocksdb_write_sync", |b| {
        b.to_async(&rt).iter(|| async {
            db_write(&db);
        })
    });
    group.bench_function("rocksdb_write_async", |b| {
        b.to_async(&rt).iter(|| {
            let db_c = Arc::clone(&db);
            tokio::task::spawn_blocking(move || db_write(&db_c))
        })
    });
    group.finish();

    clean_up();
}

criterion_group!(benches, rocksdb_benchmark,);
criterion_main!(benches);
