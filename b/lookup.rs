use criterion::{criterion_group, criterion_main, Criterion};
use freedesktop_icon_lookup::Cache;

pub fn load(c: &mut Criterion) {
    c.bench_function("Cache load Adwaita", |b| {
        b.iter(|| Cache::new().unwrap().load("Adwaita").unwrap())
    });
}

pub fn lookup(c: &mut Criterion) {
    let mut cache = Cache::new().unwrap();
    cache.load("Adwaita").unwrap();
    c.bench_function("Lookup firefox in Adwaita", |b| {
        b.iter(|| cache.lookup("firefox", "Adwaita"))
    });
}

criterion_group!(benches, load, lookup);
criterion_main!(benches);
