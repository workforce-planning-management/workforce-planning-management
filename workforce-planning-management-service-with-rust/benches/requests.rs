//! Criterion benchmarks for what a request pays apart from the database (WPM-T527).
//!
//! The domain crate's `benches/rules.rs` measures the rules a view runs once per worker; this file measures the work around them: the read
//! classification every `GET` runs before its handler, the directory search and its JSON answer, the CSV import and
//! export, and the hash a link token is looked up by. `cargo bench --bench requests` measures; `cargo test --bench
//! requests` runs each once, as CI does. The sizes are the load test's (1,000, 10,000, 50,000 and the supported 200,000). Every input is synthetic
//! and made from an index, so two runs measure the same work. Results go in `BENCHMARKS.md`.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use uuid::Uuid;
use workforce_planning_management::rules::{
    access, csv,
    directory::{self, Entry},
    link_token,
};

const SIZES: [usize; 4] = [1_000, 10_000, 50_000, 200_000];

const DEPARTMENTS: [&str; 5] = ["Wards", "Theatres", "Pharmacy", "Estates", "Finance"];
const TITLES: [&str; 4] = ["Staff nurse", "Ward clerk", "Porter", "Analyst"];

fn count(n: usize) -> u64 {
    u64::try_from(n).expect("a size fits in u64")
}

fn pid(i: usize) -> Uuid {
    Uuid::from_u128(
        0x0000_0000_0000_4000_8000_0000_0000_0000
            | u128::try_from(i).expect("an index fits in u128"),
    )
}

/// The read classification on every `GET`. `READS` is scanned in order, so the cost depends on where a path's pattern is:
/// the first, one near the end, and a path no pattern matches (the whole list, then `None`).
fn bench_classify_read(c: &mut Criterion) {
    let plan = format!("/api/workforce-plans/{}/target-organization", pid(7));
    let paths = [
        ("first", "/api/ai-register".to_string()),
        ("near_the_end", plan),
        ("no_match", "/api/no-such-route/at-all".to_string()),
    ];
    let mut group = c.benchmark_group("access/classify_read");
    for (name, path) in &paths {
        group.bench_with_input(BenchmarkId::from_parameter(name), path, |b, path| {
            b.iter(|| access::classify_read(black_box(path)));
        });
    }
    group.finish();
}

/// A directory: five departments, four titles, one in ten with no location, one in seven away and covered, one in
/// twenty on call.
fn entries(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|i| Entry {
            pid: pid(i),
            display_name: format!("Person {i:06} Surname{}", i % 997),
            job_title: TITLES[i % TITLES.len()].to_string(),
            department: DEPARTMENTS[i % DEPARTMENTS.len()].to_string(),
            location: (i % 10 != 0).then(|| format!("Site {}", i % 12)),
            organization_ref: "organization:00000000-0000-4000-8000-0000000000aa".to_string(),
            manager_name: (i % 30 != 0).then(|| format!("Manager {}", i / 30)),
            away_today: i % 7 == 0,
            covered_by: (i % 7 == 0).then(|| format!("Person {:06}", i + 1)),
            on_call: if i % 20 == 0 {
                vec!["Night site cover".to_string()]
            } else {
                Vec::new()
            },
        })
        .collect()
}

/// The directory search (`GET /api/directory`): two terms that one person in twenty matches, in one department; and an
/// empty query, where every entry is kept and sorted. The entries are cloned in each iteration, as the controller hands
/// the rule a fresh list.
fn bench_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory/search");
    for n in SIZES {
        let all = entries(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(
            BenchmarkId::new("two_terms_one_department", n),
            &all,
            |b, all| {
                b.iter(|| directory::search(black_box(all.clone()), "staff nurse", Some("wards")));
            },
        );
        group.bench_with_input(BenchmarkId::new("empty_query", n), &all, |b, all| {
            b.iter(|| directory::search(black_box(all.clone()), "", None));
        });
    }
    group.finish();

    // The answer's JSON, at the same sizes: what a whole-organization list costs to write out once it is found.
    let mut group = c.benchmark_group("directory/json");
    for n in SIZES {
        let all = entries(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &all, |b, all| {
            b.iter(|| serde_json::to_vec(black_box(all)).expect("entries serialise"));
        });
    }
    group.finish();
}

/// An import file like ESCO's: a header, then rows of five fields, one in four quoted with a comma inside and one in
/// eight with a doubled quote.
fn csv_text(n: usize) -> String {
    let mut text =
        String::from("\u{feff}conceptUri,preferredLabel,altLabels,description,status\r\n");
    for i in 0..n {
        let label = if i % 4 == 0 {
            format!("\"Skill {i}, applied\"")
        } else {
            format!("Skill {i}")
        };
        let description = if i % 8 == 0 {
            format!("\"Uses the \"\"{i}\"\" method\"")
        } else {
            format!("Plain description {i}")
        };
        text.push_str(&format!(
            "http://data.europa.eu/esco/skill/{i},{label},alt {i},{description},released\r\n"
        ));
    }
    text
}

/// The CSV import (`parse_csv`) and export (`row`, once for each row).
fn bench_csv(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv/parse");
    for n in SIZES {
        let text = csv_text(n);
        group.throughput(Throughput::Bytes(count(text.len())));
        group.bench_with_input(BenchmarkId::from_parameter(n), &text, |b, text| {
            b.iter(|| csv::parse_csv(black_box(text)).expect("the text parses"));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("csv/write");
    for n in SIZES {
        let rows = csv::parse_csv(&csv_text(n)).expect("the text parses");
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &rows, |b, rows| {
            b.iter(|| {
                rows.iter()
                    .map(|r| csv::row(black_box(r)))
                    .map(|line| line.len())
                    .sum::<usize>()
            });
        });
    }
    group.finish();
}

/// The SHA-256 a link token is stored and looked up by, once a request.
fn bench_link_token(c: &mut Criterion) {
    let token = link_token::encode([7; link_token::TOKEN_BYTES]);
    c.bench_function("link_token/hash", |b| {
        b.iter(|| link_token::hash(black_box(&token)))
    });
}

criterion_group!(
    benches,
    bench_classify_read,
    bench_directory,
    bench_csv,
    bench_link_token
);
criterion_main!(benches);
