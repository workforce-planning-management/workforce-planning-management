//! Criterion benchmarks for the pure rules a view runs once per worker (WPM-T495).
//!
//! `cargo bench -p workforce-planning-management-domain --bench rules` measures (from the service's directory); `cargo test -p workforce-planning-management-domain --bench rules` runs each once, as CI does, so a benchmark that
//! panics or no longer builds fails there. The sizes are the load test's (`scripts/load-test.sh`): 1,000, 10,000 and 50,000
//! workers, plus the supported 200,000-worker organization. Every input is synthetic and deterministic, made from the worker's index, so two runs measure the same work.
//! Results, and what they do and do not show, go in `BENCHMARKS.md`.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jiff::{Timestamp, civil::Date};
use std::hint::black_box;
use uuid::Uuid;
use workforce_planning_management_domain::{
    capacity::{self, Member, WorkerFacts},
    duplicates::{self, Known, Query},
    engagement::Basis,
    pay_gap::{self, Declared, Variable, VariableBasis, WorkerPay},
    planning::{self, Person},
    working_time,
};

const SIZES: [usize; 4] = [1_000, 10_000, 50_000, 200_000];

fn date(y: i32, m: u32, d: u32) -> Date {
    workforce_planning_management_domain::dates::ymd(y, m, d).expect("a valid date")
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0000 | n)
}

fn count(n: usize) -> u64 {
    u64::try_from(n).expect("a size fits in u64")
}

fn small(i: usize, modulus: usize) -> i32 {
    i32::try_from(i % modulus).expect("a small remainder fits in i32")
}

/// A capacity pool's workers: one role in four, one skill at 1 to 5, a third on a week's leave in the month, one in fifty leaving.
fn capacity_workers(n: usize) -> Vec<WorkerFacts> {
    let month = date(2026, 6, 1);
    (0..n)
        .map(|i| WorkerFacts {
            fte_percent: if i % 5 == 0 { 60 } else { 100 },
            hired_on: date(2020, 1, 1)
                + workforce_planning_management_domain::dates::days(i64::from(small(i, 1_500))),
            terminated_on: (i % 50 == 0).then(|| date(2026, 6, 15)),
            role_profiles: if i % 4 == 0 { vec![id(1)] } else { Vec::new() },
            skills: vec![(id(2), 1 + small(i, 5))],
            approved_leave: if i % 3 == 0 {
                let start = month
                    + workforce_planning_management_domain::dates::days(i64::from(small(i, 20)));
                vec![(
                    start,
                    start + workforce_planning_management_domain::dates::days(4),
                )]
            } else {
                Vec::new()
            },
        })
        .collect()
}

fn bench_capacity(c: &mut Criterion) {
    let members = [
        Member::RoleProfile(id(1)),
        Member::Skill {
            skill: id(2),
            min_proficiency: 3,
        },
    ];
    let month = date(2026, 6, 1);
    let mut group = c.benchmark_group("capacity/pool_supply_centi");
    for n in SIZES {
        let workers = capacity_workers(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &workers, |b, workers| {
            b.iter(|| {
                capacity::pool_supply_centi(black_box(&members), 1_000, black_box(workers), month)
            });
        });
    }
    group.finish();
}

/// A pay gap population: four in ten declared female, four male, one other, one not declared; five categories; a salary
/// spread over a range; a variable pay known for most, unknown for one in seven.
fn pay_workers(n: usize) -> Vec<WorkerPay> {
    (0..n)
        .map(|i| WorkerPay {
            key: format!("worker-{i}"),
            sex: match i % 10 {
                0..=3 => Some(Declared::Female),
                4..=7 => Some(Declared::Male),
                8 => Some(Declared::Other),
                _ => None,
            },
            category: Some(format!("band-{}", i % 5)),
            basic_minor: Some(2_500_000 + i64::from(small(i, 4_000)) * 1_000),
            variable: if i % 7 == 0 {
                Variable::Unknown
            } else {
                Variable::Known(i64::from(small(i, 50)) * 10_000)
            },
        })
        .collect()
}

fn bench_pay_gap(c: &mut Criterion) {
    let mut group = c.benchmark_group("pay_gap/report");
    for n in SIZES {
        let workers = pay_workers(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &workers, |b, workers| {
            b.iter(|| {
                pay_gap::report(black_box(workers), 10, VariableBasis::AllKnown).expect("a report")
            });
        });
    }
    group.finish();
}

/// A workforce plan's people: mostly permanent, some fixed term with an end date, some contractors and interns.
fn plan_people(n: usize) -> Vec<Person> {
    (0..n)
        .map(|i| {
            let (basis, ends_on) = match i % 10 {
                0..=6 => (Basis::Permanent, None),
                7 => (
                    Basis::FixedTerm,
                    Some(
                        date(2027, 1, 1)
                            + workforce_planning_management_domain::dates::days(i64::from(small(
                                i, 700,
                            ))),
                    ),
                ),
                8 => (
                    Basis::Contractor,
                    Some(
                        date(2026, 9, 1)
                            + workforce_planning_management_domain::dates::days(i64::from(small(
                                i, 400,
                            ))),
                    ),
                ),
                _ => (Basis::Intern, None),
            };
            Person {
                basis,
                fte_percent: if i % 6 == 0 { 50 } else { 100 },
                ends_on,
            }
        })
        .collect()
}

fn bench_planning(c: &mut Criterion) {
    let mut group = c.benchmark_group("planning/project_by_basis");
    for n in SIZES {
        let people = plan_people(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &people, |b, people| {
            b.iter(|| {
                planning::project_by_basis(
                    black_box(people),
                    date(2026, 1, 1),
                    date(2027, 6, 30),
                    Some(1_000),
                    "stated",
                )
            });
        });
    }
    group.finish();
    let mut group = c.benchmark_group("planning/contract_fte");
    for n in SIZES {
        let people = plan_people(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &people, |b, people| {
            b.iter(|| planning::contract_fte(black_box(people)));
        });
    }
    group.finish();
}

/// Twenty shifts a worker over four weeks; one worker in a hundred starts some shifts after only eight hours' rest.
fn shifts(n: usize) -> Vec<(usize, working_time::Shift)> {
    let base: Timestamp = workforce_planning_management_domain::dates::utc(2026, 6, 1, 8, 0, 0)
        .expect("a valid instant");
    let mut out = Vec::with_capacity(n * 20);
    for worker in 0..n {
        for day in 0..20_i64 {
            let early = worker % 100 == 0 && day % 5 == 1;
            let start = base + workforce_planning_management_domain::dates::days(day)
                - if early {
                    jiff::SignedDuration::from_hours(8)
                } else {
                    jiff::SignedDuration::ZERO
                };
            out.push((worker, (start, start + jiff::SignedDuration::from_hours(8))));
        }
    }
    out
}

fn bench_working_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("working_time/breaches_by_worker");
    for n in SIZES {
        let assignments = shifts(n);
        group.throughput(Throughput::Elements(count(assignments.len())));
        group.bench_with_input(
            BenchmarkId::from_parameter(n),
            &assignments,
            |b, assignments| {
                b.iter(|| working_time::breaches_by_worker(black_box(assignments), |_| Some(660)));
            },
        );
    }
    group.finish();
}

/// The people a new worker is screened against: names from a small pool of given and family names, so many share a token.
fn known(n: usize) -> Vec<Known> {
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
    (0..n)
        .map(|i| Known {
            pid: id(u128::try_from(i).expect("an index fits in u128") + 100),
            display_name: format!("{} {} {}", GIVEN[i % 12], FAMILY[(i / 12) % 12], i / 144),
            organization_ref: "organization:00000000-0000-4000-8000-0000000000aa".to_string(),
            worker_number: format!("W{i}"),
        })
        .collect()
}

fn bench_duplicates(c: &mut Criterion) {
    let query = Query {
        display_name: "Kathy Johnson".to_string(),
        organization_ref: "organization:00000000-0000-4000-8000-0000000000aa".to_string(),
        worker_number: Some("W4000".to_string()),
    };
    let mut group = c.benchmark_group("duplicates/possible_duplicates");
    for n in SIZES {
        let held = known(n);
        group.throughput(Throughput::Elements(count(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &held, |b, held| {
            b.iter(|| duplicates::possible_duplicates(black_box(&query), black_box(held)));
        });
    }
    group.finish();
}

criterion_group!(
    rules,
    bench_capacity,
    bench_pay_gap,
    bench_planning,
    bench_working_time,
    bench_duplicates
);
criterion_main!(rules);
