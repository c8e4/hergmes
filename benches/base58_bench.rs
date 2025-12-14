use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use hergmes::address::base58;

fn make_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|i| ((i * 7 + 13) % 256) as u8).collect()
}

const ERGO_ADDRS: &[(&str, &str)] = &[
    (
        "p2pk_mainnet",
        "9fRusAarL1KkrWQVsxSRVYnvWxaAT2A96cKtNn9tvPh5XUyCisr",
    ),
    ("p2sh_mainnet", "8sZ2fVu5VUQKEmWt4xRRDBYzuw5aevhhziPBDGB"),
    (
        "p2s_fee_mainnet",
        "2iHkR7CWvD1R4j1yZg5bkeDRQavjAaVPeTDFGGLZduHyfWMuYpmhHocX8GJoaieTx78FntzJbCBVL6rf96ocJoZdmWBL2fci7NqWgAirppPQmZ7fN9V6z13Ay6brPriBKYqLp1bT2Fk4FkFLCfdPpe",
    ),
    (
        "p2s_long_mainnet",
        "2Z4YBkDsDvQj8BX7xiySFewjitqp2ge9c99jfes2whbtKitZTxdBYqbrVZUvZvKv6aqn9by4kp3LE1c26LCyosFnVnm6b6U1JYvWpYmL2ZnixJbXLjWAWuBThV1D6dLpqZJYQHYDznJCk49g5TUiS4q8khpag2aNmHwREV7JSsypHdHLgJT7MGaw51aJfNubyzSKxZ4AJXFS27EfXwyCLzW1K6GVqwkJtCoPvrcLqmqwacAWJPkmh78nke9H4oT88XmSbRt2n9aWZjosiZCafZ4osUDxmZcc5QVEeTWn8drSraY3eFKe8Mu9MSCcVU",
    ),
];

fn bench_encode_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("base58_encode_sizes");

    for &n in &[32usize, 64, 256, 1024, 4096] {
        let data = make_bytes(n);
        group.throughput(Throughput::Bytes(n as u64));

        group.bench_with_input(BenchmarkId::new("hergmes", n), &data, |b, d| {
            b.iter(|| black_box(base58::encode(black_box(d))))
        });

        group.bench_with_input(BenchmarkId::new("bs58", n), &data, |b, d| {
            b.iter(|| black_box(bs58::encode(black_box(d)).into_string()))
        });
    }

    group.finish();
}

fn bench_decode_into_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("base58_decode_into_sizes");

    for &n in &[32usize, 64, 256, 1024, 4096] {
        let data = make_bytes(n);
        let encoded = base58::encode(&data);

        group.throughput(Throughput::Bytes(encoded.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("hergmes_decode_into_reuse", n),
            &encoded,
            |b, s| {
                let mut out = vec![0u8; n + 64];
                b.iter(|| black_box(base58::decode_into(black_box(s), &mut out)))
            },
        );

        group.bench_with_input(BenchmarkId::new("bs58_onto_reuse", n), &encoded, |b, s| {
            let mut out = vec![0u8; n + 64];
            b.iter(|| black_box(bs58::decode(black_box(s)).onto(&mut out[..]).unwrap()))
        });
    }

    group.finish();
}

fn bench_decode_into_ergo_addrs(c: &mut Criterion) {
    let mut group = c.benchmark_group("base58_decode_into_ergo_addrs");

    for (name, addr) in ERGO_ADDRS {
        group.throughput(Throughput::Bytes(addr.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("hergmes_decode_into_reuse", *name),
            addr,
            |b, s| {
                let mut out = [0u8; 8192];
                b.iter(|| black_box(base58::decode_into(black_box(s), &mut out)))
            },
        );

        group.bench_with_input(BenchmarkId::new("bs58_onto_reuse", *name), addr, |b, s| {
            let mut out = [0u8; 8192];
            b.iter(|| black_box(bs58::decode(black_box(s)).onto(&mut out[..]).unwrap()))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encode_sizes,
    bench_decode_into_sizes,
    bench_decode_into_ergo_addrs
);
criterion_main!(benches);
