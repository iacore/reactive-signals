use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

pub fn discard_scopes_with_signals(c: &mut Criterion) {
    c.bench_function("Discard 1,000 nested scopes with 1 func signal each", |b| {
        b.iter_batched(
            reactor::profile::create_1000_nested,
            |(scope, _start, _end)| scope.discard(),
            BatchSize::SmallInput,
        );
    });

    c.bench_function(
        "Discard 1,000 sibling scopes with 1 func signal each",
        |b| {
            b.iter_batched(
                reactor::profile::create_1000_nested,
                |(scope, _, _)| scope.discard(),
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(benches, discard_scopes_with_signals,);

criterion_main!(benches);
