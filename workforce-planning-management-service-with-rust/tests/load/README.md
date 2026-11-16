# WPM Load Testing Suite

This directory contains the **distributed load testing** infrastructure for Workforce Planning Management.

## Overview

The load testing suite provides comprehensive testing capabilities:

1. **Distributed Load Testing** - Run tests across multiple machines/workers
2. **Long-Running Tests** - Endurance and soak tests that run for hours/days
3. **Multiple Test Types**:
   - **Load Tests** - Steady load at target RPS
   - **Stress Tests** - Increasing load until failure
   - **Spike Tests** - Sudden load spikes
   - **Soak Tests** - Sustained load over extended periods
   - **Endurance Tests** - Long-running steady load

## Quick Start

### Prerequisites

- Rust 1.99+ (stable)
- PostgreSQL 18 (for test database)
- WPM service running

### Run a Basic Load Test

```bash
cd workforce-planning-management-service-with-rust
cargo run --release --manifest-path tests/load/Cargo.toml -- --help
```

### Examples

```bash
# Quick load test (60 seconds, 100 RPS, 4 workers)
cargo run --release --manifest-path tests/load/Cargo.toml -- run -d 60 -r 100

# Distributed test (4 workers, 300 seconds, 500 RPS)
cargo run --release --manifest-path tests/load/Cargo.toml -- distributed -w 4 -d 300 -r 500

# Endurance test (8 hours at 50 RPS)
cargo run --release --manifest-path tests/load/Cargo.toml -- endurance -d 28800 -r 50

# Stress test (100 to 5000 RPS in increments of 100)
cargo run --release --manifest-path tests/load/Cargo.toml -- stress -s 100 -m 5000 -i 100

# Spike test (base 100 RPS, spike to 2000 RPS, 5 spikes)
cargo run --release --manifest-path tests/load/Cargo.toml -- spike -b 100 -s 2000 -k 5

# Soak test (24 hours at 100 RPS)
cargo run --release --manifest-path tests/load/Cargo.toml -- soak -d 86400 -r 100
```

## Test Scenarios

The suite supports multiple test scenarios:

| Scenario | Description | Use Case |
|----------|-------------|----------|
| `mixed` | Mixed read/write/auth requests | General performance testing |
| `read-heavy` | Mostly GET requests | Read performance testing |
| `write-heavy` | Mostly POST/PUT requests | Write performance testing |
| `auth-heavy` | Authentication-focused | Auth performance testing |
| `api-heavy` | API endpoint focused | API performance testing |
| `stress` | Heavy requests | Stress testing |

### Specify Scenario

```bash
# Use read-heavy scenario
cargo run --release --manifest-path tests/load/Cargo.toml -- run -s read-heavy

# Use write-heavy scenario
cargo run --release --manifest-path tests/load/Cargo.toml -- run -s write-heavy
```

## Configuration

### Environment Variables

```bash
# Service URL (default: http://localhost:1241)
export WPM_BASE_URL=http://localhost:1241

# Test duration in seconds (default: 60)
export WPM_LOAD_TEST_DURATION=300

# Target RPS (default: 100)
export WPM_TARGET_RPS=500

# Number of workers (default: 4)
export WPM_LOAD_TEST_WORKERS=8

# Maximum requests per worker (default: 10000)
export WPM_MAX_REQUESTS=100000

# Warmup seconds (default: 10)
export WPM_WARMUP_SECONDS=30

# Rampup seconds (default: 20)
export WPM_RAMPUP_SECONDS=60

# Rampdown seconds (default: 10)
export WPM_RAMPDOWN_SECONDS=30

# Scenario (default: mixed)
export WPM_SCENARIO=mixed
```

## Output

### Console Output

```
================================================================================
LOAD TEST SUMMARY
================================================================================

Total Duration: 60.00s
Total Requests: 24000
Total Success: 23950
Total Errors: 50
Success Rate: 99.79%

--------------------------------------------------------------------------------
PHASE METRICS
--------------------------------------------------------------------------------

Phase: warmup
  Requests: 1200
  Success: 1198 (99.83%)
  Errors: 2
  Avg Latency: 15.23ms
  P50 Latency: 14.56ms
  P95 Latency: 25.89ms
  P99 Latency: 45.12ms

Phase: rampup
  Requests: 2400
  Success: 2395 (99.79%)
  Errors: 5
  ...
```

### JSON Output

```bash
# Enable JSON output
cargo run --release --manifest-path tests/load/Cargo.toml -- run --json
```

```json
{
  "total_duration_secs": 60.0,
  "total_requests": 24000,
  "total_success": 23950,
  "total_errors": 50,
  "success_rate": 99.79,
  "phases": {
    "warmup": {
      "total_requests": 1200,
      "success_count": 1198,
      "error_count": 2,
      "success_rate": 99.83,
      "avg_latency_ms": 15.23,
      "p50_latency_ms": 14.56,
      "p95_latency_ms": 25.89,
      "p99_latency_ms": 45.12,
      "status_codes": {
        "200": 1198,
        "500": 2
      }
    },
    ...
  }
}
```

## Distributed Testing

For true distributed testing across multiple machines:

### Option 1: Using k6 (Recommended for Production)

Install k6:
```bash
# macOS
brew install k6

# Linux
sudo apt install k6

# Windows (chocolatey)
choco install k6
```

Create a k6 script (see `k6/` directory):
```bash
k6 run -e WPM_BASE_URL=http://localhost:1241 k6/load-test.js
```

### Option 2: Distributed Rust Workers

The Rust load tester can coordinate multiple workers:

```bash
# On each worker machine, run in worker mode
cargo run --release --manifest-path tests/load/Cargo.toml -- distributed-worker \
  --worker-id 1 \
  --coordinator http://coordinator:8080 \
  --base-url http://wpm-service:1241
```

## Long-Running Tests

### Endurance Tests

Test the system's ability to sustain load over extended periods:

```bash
# 8 hour endurance test at 50 RPS
cargo run --release --manifest-path tests/load/Cargo.toml -- endurance -d 28800 -r 50

# 24 hour endurance test at 100 RPS
cargo run --release --manifest-path tests/load/Cargo.toml -- endurance -d 86400 -r 100
```

### Soak Tests

Test for memory leaks and resource exhaustion:

```bash
# 24 hour soak test with memory checking
cargo run --release --manifest-path tests/load/Cargo.toml -- soak -d 86400 -r 100 --check-memory
```

## Test Results Interpretation

### Key Metrics

| Metric | Target | Interpretation |
|--------|--------|----------------|
| Success Rate | > 99.9% | Percentage of successful requests |
| P50 Latency | < 100ms | Median response time |
| P95 Latency | < 500ms | 95th percentile response time |
| P99 Latency | < 1000ms | 99th percentile response time |
| Error Rate | < 0.1% | Percentage of failed requests |

### Common Issues

1. **High P99 Latency** - Some requests are taking too long
2. **Low Success Rate** - System is returning errors
3. **Increasing Latency** - System is degrading under load
4. **Memory Growth** - Potential memory leak
5. **Connection Errors** - Database/connection pool issues

## Integration with CI/CD

Add load tests to your CI pipeline:

```yaml
# .github/workflows/load-test.yml
name: Load Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  load-test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:18
        env:
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
      - run: cargo build --release --manifest-path tests/load/Cargo.toml
      - run: |
          cargo run --release --manifest-path tests/load/Cargo.toml -- \
            run -d 60 -r 100 -w 4 --json > load-test-results.json
      - uses: actions/upload-artifact@v4
        with:
          name: load-test-results
          path: load-test-results.json
```

## Best Practices

1. **Start Small** - Begin with low RPS and increase gradually
2. **Monitor Resources** - Watch CPU, memory, disk, and network usage
3. **Test in Staging** - Don't run load tests against production
4. **Use Realistic Data** - Test with realistic dataset sizes
5. **Test Regularly** - Run load tests after major changes
6. **Document Results** - Track performance over time
7. **Set Baselines** - Know your expected performance

## Troubleshooting

### Common Problems

1. **Connection Refused** - Service not running or wrong URL
   - Check `WPM_BASE_URL`
   - Verify service is running: `curl http://localhost:1241/_health/live`

2. **High Error Rate** - Service returning errors
   - Check service logs
   - Verify database is running
   - Check connection pool settings

3. **Timeout Errors** - Requests timing out
   - Increase timeout in config
   - Check for slow queries
   - Verify database performance

4. **Low RPS** - Not reaching target RPS
   - Check worker count
   - Verify system resources
   - Check for bottlenecks

### Debug Mode

```bash
# Enable verbose logging
cargo run --release --manifest-path tests/load/Cargo.toml -- --verbose run
```

## Performance Targets

Based on WPM's design goals:

| Scenario | Target RPS | Target P95 | Target P99 |
|----------|-----------|------------|------------|
| Small (1k workers) | 100+ | < 100ms | < 500ms |
| Medium (10k workers) | 500+ | < 200ms | < 1000ms |
| Large (50k workers) | 1000+ | < 500ms | < 2000ms |

## Contributing

1. Add new test scenarios in `src/scenarios.rs`
2. Add new metrics in `src/metrics.rs`
3. Update documentation in `README.md`
4. Test with different configurations

## License

BSD 3-Clause License (same as WPM project)
