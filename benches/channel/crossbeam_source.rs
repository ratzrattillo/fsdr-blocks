use criterion::{Criterion, criterion_group, criterion_main};
use fsdr_blocks::channel::CrossbeamSource;
use futuresdr::runtime::mocker::{Mocker, Writer};
use rand::RngExt;

/// This benchmark seems to highly depend on the underlying scheduling of polling from the channel
// cargo bench --profile release --bench crossbeam_source --features="crossbeam"
pub fn crossbeam_source_boxed_slice_u32(c: &mut Criterion) {
    let n_samp = 8192;
    let input: Vec<u32> = rand::rng()
        .sample_iter(rand::distr::Uniform::<u32>::new(0, 1024).unwrap())
        .take(n_samp)
        .collect();

    let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();

    let mut group = c.benchmark_group("crossbeam_source");

    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function("mock-u32-crossbeam-source", |b| {
        b.iter(|| {
            let block: CrossbeamSource<u32, Writer<u32>> = CrossbeamSource::new(rx.clone());
            let mut mocker = Mocker::new(block);

            tx.try_send(input.clone().into_boxed_slice()).unwrap();

            mocker.output().reserve(n_samp);
            mocker.run();
        });
    });

    group.finish();
}

criterion_group!(benches, crossbeam_source_boxed_slice_u32);
criterion_main!(benches);
