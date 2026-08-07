use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use quark::{Registrable, Registry};

struct BenchDescriptor {
    key: String,
    #[allow(dead_code)] // retained for realistic descriptor shape in benches
    index: usize,
}

impl Registrable for BenchDescriptor {
    fn registry_key(&self) -> &str {
        &self.key
    }
}

fn build_registry(n: usize) -> Registry<BenchDescriptor> {
    let mut reg = Registry::new();
    for i in 0..n {
        let desc = Box::leak(Box::new(BenchDescriptor {
            key: format!("item_{:05}", i),
            index: i,
        }));
        reg.register(desc);
    }
    reg
}

fn bench_sizes() -> &'static [usize] {
    // register_build leaks Box per iteration; 10_000 OOMs GitHub-hosted runners.
    if std::env::var_os("CI").is_some() {
        &[1, 10, 100, 1000]
    } else {
        &[1, 10, 100, 1000, 10_000]
    }
}

fn bench_register_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("register_build");
    for &n in bench_sizes() {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut reg = Registry::<BenchDescriptor>::new();
                for i in 0..n {
                    let desc = Box::leak(Box::new(BenchDescriptor {
                        key: format!("item_{:05}", i),
                        index: i,
                    }));
                    reg.register(desc);
                }
                black_box(reg)
            });
        });
    }
    group.finish();
}

fn bench_get_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_hit");
    for &n in bench_sizes() {
        let reg = build_registry(n);
        let key = format!("item_{:05}", n / 2);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(reg.get(&key)));
        });
    }
    group.finish();
}

fn bench_get_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_miss");
    for &n in bench_sizes() {
        let reg = build_registry(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(reg.get("missing_key")));
        });
    }
    group.finish();
}

fn bench_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("list");
    for &n in bench_sizes() {
        let reg = build_registry(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(reg.list()));
        });
    }
    group.finish();
}

fn bench_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter");
    for &n in bench_sizes() {
        let reg = build_registry(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(reg.iter().count()));
        });
    }
    group.finish();
}

fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");
    for &n in bench_sizes() {
        let reg = build_registry(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(reg.clone()));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_register_build,
    bench_get_hit,
    bench_get_miss,
    bench_list,
    bench_iter,
    bench_clone
);
criterion_main!(benches);
