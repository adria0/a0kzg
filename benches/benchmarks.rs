use a0kzg::{Kzg, Scalar};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use once_cell::sync::Lazy;
use rand::Rng;

static ZKGS: Lazy<Vec<Kzg>> = Lazy::new(|| {
    [32, 64, 128, 256]
        .iter()
        .map(|n| Kzg::trusted_setup(1 + *n as usize))
        .collect::<Vec<_>>()
});

static POINTS: Lazy<Vec<(Scalar, Scalar)>> = Lazy::new(|| {
    let mut rng = rand::thread_rng();
    (0..1025)
        .map(|_| {
            let rnd_x: [u64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            let rnd_y: [u64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
            (Scalar::from_raw(rnd_x), Scalar::from_raw(rnd_y))
        })
        .collect::<Vec<_>>()
});

fn zkg_prover(c: &mut Criterion) {
    let mut group = c.benchmark_group("zkg_prover");
    for zkg in ZKGS.iter() {
        let size = zkg.max_degree();
        let (p, _c) = zkg.poly_commitment_from_set(&POINTS[0..size]);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| zkg.prove(&p, &POINTS[0..size / 2]))
        });
    }
    group.finish();
}

fn zkg_verifier(c: &mut Criterion) {
    let mut group = c.benchmark_group("zkg_verifier");
    for zkg in ZKGS.iter() {
        let size = zkg.max_degree();
        let (p, c) = zkg.poly_commitment_from_set(&POINTS[0..size]);
        let pi = zkg.prove(&p, &POINTS[0..size / 2]);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| zkg.verify(&c, &POINTS[0..size / 2], &pi))
        });
    }
    group.finish();
}

criterion_group!(benches, zkg_prover, zkg_verifier);
criterion_main!(benches);
