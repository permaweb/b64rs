use b64rs::{decode_bytes, encode_bytes};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_codec(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec");

    for size in [32usize, 1024, 64 * 1024] {
        let input = vec![0x5a; size];
        let encoded = encode_bytes(&input);
        let encoded_standard = base64_simd::STANDARD.encode_to_string(&input).into_bytes();
        let mut encoded_with_spaces = encoded_standard.clone();
        if encoded_with_spaces.len() > 8 {
            encoded_with_spaces.insert(4, b' ');
        }

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("encode_{size}b"), |b| {
            b.iter(|| encode_bytes(black_box(&input)))
        });
        group.bench_function(format!("decode_urlsafe_{size}b"), |b| {
            b.iter(|| decode_bytes(black_box(&encoded)).unwrap())
        });
        group.bench_function(format!("decode_standard_{size}b"), |b| {
            b.iter(|| decode_bytes(black_box(&encoded_standard)).unwrap())
        });
        group.bench_function(format!("decode_standard_ws_{size}b"), |b| {
            b.iter(|| decode_bytes(black_box(&encoded_with_spaces)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_codec);
criterion_main!(benches);
