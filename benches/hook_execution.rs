//! Benchmark for hook execution performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use std::time::Duration;

fn benchmark_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("fasthooks --version", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--release", "--quiet", "--", "--version"])
                .output()
                .expect("Failed to run fasthooks");
            black_box(output)
        })
    });

    group.finish();
}

fn benchmark_config_parsing(c: &mut Criterion) {
    let config_content = r#"
version = "1"

[settings]
parallel = true

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "lint"
run = "echo lint"
glob = "*.rs"

[[hooks.pre-commit.tasks]]
name = "format"
run = "echo format"
glob = "*.rs"

[[hooks.pre-commit.tasks]]
name = "test"
run = "echo test"
"#;

    c.bench_function("parse_config", |b| {
        b.iter(|| {
            let _config: toml::Value = toml::from_str(black_box(config_content)).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_startup, benchmark_config_parsing);
criterion_main!(benches);
