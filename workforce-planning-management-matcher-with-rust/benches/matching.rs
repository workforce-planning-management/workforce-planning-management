//! Criterion benchmarks for the matcher (WPM-T495).
//!
//! `cargo bench --bench matching` measures; `cargo test --bench matching` runs each once, as CI does. Screening one incoming
//! record against everyone held is the import's inner loop, so it is measured at 1,000, 10,000 and 50,000 held records. Every
//! record is synthetic and made from its index, so two runs measure the same work.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use workforce_planning_management_matcher::{MatchConfig, MatchingEngine, WorkerRecord, scorer};

const SIZES: [usize; 3] = [1_000, 10_000, 50_000];

const GIVEN: [&str; 12] = [
    "Ada",
    "Grace",
    "Alan",
    "Katherine",
    "Maria",
    "Wei",
    "Amara",
    "Sven",
    "Priya",
    "Tomás",
    "Yuki",
    "Olu",
];
const FAMILY: [&str; 12] = [
    "Lovelace", "Hopper", "Turing", "Johnson", "Silva", "Zhang", "Okafor", "Larsen", "Patel",
    "García", "Sato", "Adeyemi",
];

/// A held record: a name from a small pool (so many share a token), an email, a date of birth for most, a worker number.
fn held(i: usize) -> WorkerRecord {
    let given = GIVEN[i % 12];
    let family = FAMILY[(i / 12) % 12];
    let record = WorkerRecord::new()
        .with_given_name(given)
        .with_family_name(family)
        .with_email(format!(
            "{}.{}{}@example.org",
            given.to_lowercase(),
            family.to_lowercase(),
            i
        ))
        .with_worker_number("organization:acme", format!("E-{i}"));
    let year = 1960 + i32::try_from(i % 45).expect("a small remainder fits in i32");
    let month = 1 + u32::try_from(i % 12).expect("a small remainder fits in u32");
    let day = 1 + u32::try_from(i % 28).expect("a small remainder fits in u32");
    if i.is_multiple_of(5) {
        record
    } else {
        record
            .with_date_of_birth(year, month, day)
            .expect("a valid date")
    }
}

fn incoming() -> WorkerRecord {
    WorkerRecord::new()
        .with_given_name("Kathy")
        .with_family_name("Johnson")
        .with_email("K.Johnson@Example.org")
        .with_date_of_birth(1971, 4, 9)
        .expect("a valid date")
}

fn engine() -> MatchingEngine {
    MatchingEngine::new(MatchConfig::default()).expect("the default configuration is valid")
}

fn bench_pair(c: &mut Criterion) {
    let engine = engine();
    let a = incoming();
    let b = held(3);
    c.bench_function("match_workers/one_pair", |bench| {
        bench.iter(|| engine.match_workers(black_box(&a), black_box(&b)));
    });
}

fn bench_screen(c: &mut Criterion) {
    let engine = engine();
    let query = incoming();
    let mut group = c.benchmark_group("rank_one_to_many");
    for n in SIZES {
        let records: Vec<WorkerRecord> = (0..n).map(held).collect();
        group.throughput(Throughput::Elements(
            u64::try_from(n).expect("a size fits in u64"),
        ));
        group.bench_with_input(
            BenchmarkId::from_parameter(n),
            &records,
            |bench, records| {
                bench.iter(|| engine.rank_one_to_many(black_box(&query), black_box(records)));
            },
        );
    }
    group.finish();
}

fn bench_scorers(c: &mut Criterion) {
    let mut group = c.benchmark_group("scorer");
    for (a, b) in [
        ("Katherine", "Kathryn"),
        ("Adeyemi", "Adeyemmi"),
        ("Zhang", "García"),
    ] {
        let label = format!("{a}~{b}");
        group.bench_with_input(
            BenchmarkId::new("jaro_winkler", &label),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| scorer::jaro_winkler_similarity(black_box(a), black_box(b)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("levenshtein", &label),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| scorer::levenshtein_similarity(black_box(a), black_box(b)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("combined", &label),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| scorer::combined_similarity(black_box(a), black_box(b)));
            },
        );
    }
    group.finish();
}

criterion_group!(matching, bench_pair, bench_screen, bench_scorers);
criterion_main!(matching);
