use b64rs::{decode_bytes, encode_bytes};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_codec(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec");

    for size in [32usize, 1024, 64 * 1024] {
        let input = vec![0x5a; size];
        let encoded = encode_bytes(&input);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("encode_{size}b"), |b| {
            b.iter(|| encode_bytes(black_box(&input)))
        });
        group.bench_function(format!("decode_{size}b"), |b| {
            b.iter(|| decode_bytes(black_box(&encoded)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_codec);
criterion_main!(benches);
