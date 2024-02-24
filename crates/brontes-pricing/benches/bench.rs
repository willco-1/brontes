mod it;
mod state;
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(
    name = it_runs;
    config = Criterion::default().significance_level(0.1).noise_threshold(0.05).sample_size(10);
    targets = it::bench_block_pricing
);
criterion_group!(v3, state::bench_v3_price_requests, state::bench_v3_state_loads);
criterion_group!(v2, state::bench_v2_price_requests, state::bench_v2_state_loads);

criterion_main!(v2, v3, it_runs);