use criterion::{Criterion, criterion_group, criterion_main};
use fsdr_blocks::channel::CrossbeamSink;
use futuresdr::runtime::mocker::{Mocker, Reader};
use rand::RngExt;

/// This benchmark seems to highly depend on the underlying scheduling of polling from the channel
// cargo bench --profile release --bench crossbeam_sink --features="crossbeam"
pub fn crossbeam_sink_boxed_slice_u32(c: &mut Criterion) {
    let n_samp = 8192;
    let input: Vec<u32> = rand::rng()
        .sample_iter(rand::distr::Uniform::<u32>::new(0, 1024).unwrap())
        .take(n_samp)
        .collect();
    // let input = input.into_boxed_slice();
    // let input = vec![input];

    let (tx, rx) = crossbeam_channel::unbounded::<Box<[u32]>>();

    let mut group = c.benchmark_group("crossbeam_sink");

    group.throughput(criterion::Throughput::Elements(n_samp as u64));

    group.bench_function("mock-u32-crossbeam-sink", |b| {
        b.iter(|| {
            let block: CrossbeamSink<u32, Reader<u32>> = CrossbeamSink::new(tx.clone());
            let mut mocker = Mocker::new(block);

            // mocker.input(0, input.clone());
            mocker.input().set(input.clone());
            mocker.run();

            // receive again all samples sent into the crossbeam_sink...
            rx.iter().take(1).for_each(drop);
        });
    });

    group.finish();
}

criterion_group!(benches, crossbeam_sink_boxed_slice_u32);
criterion_main!(benches);
