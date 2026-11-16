//! Workforce Planning Management - Distributed Load Testing
//!
//! This binary provides comprehensive load testing capabilities for the WPM service.
//! It supports:
//! - Distributed load testing across multiple workers
//! - Long-running endurance tests
//! - Stress testing
//! - Spike testing
//! - Soak testing
//!
//! Usage:
//!   cargo run --release -- --help
//!
//! Examples:
//!   # Run a quick load test
//!   cargo run --release -- -d 60 -r 100
//!
//!   # Run distributed test with 4 workers
//!   cargo run --release -- --distributed -w 4 -d 300 -r 500
//!
//!   # Run endurance test for 8 hours
//!   cargo run --release -- --endurance -d 28800 -r 50

use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task;
use tracing::{error, info, warn};

mod config;
mod metrics;
mod scenarios;

use config::LoadTestConfig;
use metrics::LoadTestMetrics;
use scenarios::{Request, RequestGenerator, Scenario, ScenarioType, MixedScenario, ReadHeavyScenario, WriteHeavyScenario, AuthHeavyScenario, ApiHeavyScenario, StressScenario};

/// Workforce Planning Management Load Tester
#[derive(Parser, Debug)]
#[command(name = "wpm-load-test")]
#[command(author = "WPM Team")]
#[command(version = "0.1.0")]
#[command(about = "Distributed load testing for WPM service", long_about = None)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// JSON output for metrics
    #[arg(short, long)]
    json: bool,

    /// Prometheus metrics endpoint (if enabled)
    #[arg(long, default_value = "0.0.0.0:9090")]
    prometheus: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a load test with specified parameters
    Run {
        /// Base URL of the WPM service (e.g., http://localhost:1241)
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,

        /// Duration of the test in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Requests per second (target RPS)
        #[arg(short, long, default_value = "100")]
        rate: u32,

        /// Number of concurrent workers
        #[arg(short, long, default_value = "4")]
        workers: usize,

        /// Maximum number of requests per worker
        #[arg(short, long, default_value = "10000")]
        max_requests: u64,

        /// Scenario to run
        #[arg(short, long, default_value = "mixed")]
        scenario: String,

        /// Warmup period in seconds
        #[arg(long, default_value = "10")]
        warmup: u64,

        /// Ramp-up period in seconds
        #[arg(long, default_value = "20")]
        rampup: u64,

        /// Ramp-down period in seconds
        #[arg(long, default_value = "10")]
        rampdown: u64,
    },

    /// Run distributed load test across multiple machines
    Distributed {
        /// Number of worker nodes
        #[arg(short, long, default_value = "4")]
        workers: usize,

        /// Test duration per worker in seconds
        #[arg(short, long, default_value = "300")]
        duration: u64,

        /// Target requests per second
        #[arg(short, long, default_value = "500")]
        rate: u32,

        /// Base URL of the WPM service
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,
    },

    /// Run endurance test (long-running, steady load)
    Endurance {
        /// Base URL of the WPM service
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,

        /// Duration in seconds (default: 8 hours = 28800s)
        #[arg(short, long, default_value = "28800")]
        duration: u64,

        /// Steady request rate
        #[arg(short, long, default_value = "50")]
        rate: u32,

        /// Number of workers
        #[arg(short, long, default_value = "2")]
        workers: usize,
    },

    /// Run stress test (increasing load until failure)
    Stress {
        /// Base URL of the WPM service
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,

        /// Starting RPS
        #[arg(short, long, default_value = "100")]
        start_rate: u32,

        /// Maximum RPS to attempt
        #[arg(short, long, default_value = "5000")]
        max_rate: u32,

        /// RPS increment
        #[arg(short, long, default_value = "100")]
        increment: u32,

        /// Duration at each RPS level in seconds
        #[arg(short, long, default_value = "60")]
        step_duration: u64,

        /// Number of workers
        #[arg(short, long, default_value = "4")]
        workers: usize,
    },

    /// Run spike test (sudden load spikes)
    Spike {
        /// Base URL of the WPM service
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,

        /// Base RPS
        #[arg(short, long, default_value = "100")]
        base_rate: u32,

        /// Spike RPS
        #[arg(short, long, default_value = "2000")]
        spike_rate: u32,

        /// Spike duration in seconds
        #[arg(short, long, default_value = "30")]
        spike_duration: u64,

        /// Recovery period in seconds
        #[arg(short, long, default_value = "60")]
        recovery: u64,

        /// Number of spikes to test
        #[arg(short, long, default_value = "5")]
        spikes: u32,

        /// Number of workers
        #[arg(short, long, default_value = "4")]
        workers: usize,
    },

    /// Run soak test (sustained load over extended period)
    Soak {
        /// Base URL of the WPM service
        #[arg(short, long, default_value = "http://localhost:1241")]
        base_url: String,

        /// Duration in seconds (default: 24 hours = 86400s)
        #[arg(short, long, default_value = "86400")]
        duration: u64,

        /// Steady request rate
        #[arg(short, long, default_value = "100")]
        rate: u32,

        /// Number of workers
        #[arg(short, long, default_value = "4")]
        workers: usize,

        /// Check for memory leaks
        #[arg(long)]
        check_memory: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    info!("WPM Load Test v0.1.0");
    info!("Starting load test with command: {:?}", args.command);

    match args.command {
        Commands::Run {
            base_url,
            duration,
            rate,
            workers,
            max_requests,
            scenario,
            warmup,
            rampup,
            rampdown,
        } => {
            run_load_test(
                &base_url,
                duration,
                rate,
                workers,
                max_requests,
                &scenario,
                warmup,
                rampup,
                rampdown,
                args.json,
            )
            .await?;
        }
        Commands::Distributed {
            workers,
            duration,
            rate,
            base_url,
        } => {
            run_distributed_test(&base_url, duration, rate, workers, args.json).await?;
        }
        Commands::Endurance {
            base_url,
            duration,
            rate,
            workers,
        } => {
            run_endurance_test(&base_url, duration, rate, workers, args.json).await?;
        }
        Commands::Stress {
            base_url,
            start_rate,
            max_rate,
            increment,
            step_duration,
            workers,
        } => {
            run_stress_test(
                &base_url,
                start_rate,
                max_rate,
                increment,
                step_duration,
                workers,
                args.json,
            )
            .await?;
        }
        Commands::Spike {
            base_url,
            base_rate,
            spike_rate,
            spike_duration,
            recovery,
            spikes,
            workers,
        } => {
            run_spike_test(
                &base_url,
                base_rate,
                spike_rate,
                spike_duration,
                recovery,
                spikes,
                workers,
                args.json,
            )
            .await?;
        }
        Commands::Soak {
            base_url,
            duration,
            rate,
            workers,
            check_memory,
        } => {
            run_soak_test(
                &base_url,
                duration,
                rate,
                workers,
                check_memory,
                args.json,
            )
            .await?;
        }
    }

    info!("Load test completed successfully");
    Ok(())
}

async fn run_load_test(
    base_url: &str,
    duration_secs: u64,
    target_rps: u32,
    num_workers: usize,
    max_requests: u64,
    scenario: &str,
    warmup_secs: u64,
    rampup_secs: u64,
    rampdown_secs: u64,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = LoadTestConfig {
        base_url: base_url.to_string(),
        duration: Duration::from_secs(duration_secs),
        target_rps,
        num_workers,
        max_requests_per_worker: max_requests,
        warmup: Duration::from_secs(warmup_secs),
        rampup: Duration::from_secs(rampup_secs),
        rampdown: Duration::from_secs(rampdown_secs),
        scenario: scenario.parse().unwrap_or(ScenarioType::Mixed),
    };

    let metrics = Arc::new(LoadTestMetrics::new());
    
    if json_output {
        metrics.set_json_output(true);
    }

    info!(
        "Starting load test: {} RPS, {} workers, {} seconds",
        target_rps, num_workers, duration_secs
    );
    info!("Scenario: {:?}", config.scenario);

    // Warmup phase
    if warmup_secs > 0 {
        info!("Warmup phase: {} seconds", warmup_secs);
        run_phase(
            &config,
            &metrics,
            Duration::from_secs(warmup_secs),
            target_rps / 2, // Half rate during warmup
            "warmup",
        )
        .await?;
    }

    // Rampup phase
    if rampup_secs > 0 {
        info!("Rampup phase: {} seconds", rampup_secs);
        run_phase(
            &config,
            &metrics,
            Duration::from_secs(rampup_secs),
            target_rps,
            "rampup",
        )
        .await?;
    }

    // Main test phase
    info!("Main test phase: {} seconds", duration_secs);
    run_phase(
        &config,
        &metrics,
        Duration::from_secs(duration_secs),
        target_rps,
        "main",
    )
    .await?;

    // Rampdown phase
    if rampdown_secs > 0 {
        info!("Rampdown phase: {} seconds", rampdown_secs);
        run_phase(
            &config,
            &metrics,
            Duration::from_secs(rampdown_secs),
            target_rps / 2, // Half rate during rampdown
            "rampdown",
        )
        .await?;
    }

    // Print summary
    metrics.print_summary();
    
    if json_output {
        println!("{}", serde_json::to_string(&metrics.summary())?);
    }

    Ok(())
}

async fn run_phase(
    config: &LoadTestConfig,
    metrics: &Arc<LoadTestMetrics>,
    duration: Duration,
    target_rps: u32,
    phase_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let semaphore = Arc::new(Semaphore::new(target_rps as usize));
    
    let start = Instant::now();
    let end = start + duration;

    let mut handles = vec![];

    for worker_id in 0..config.num_workers {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let metrics = metrics.clone();
        let config = config.clone();
        let phase_name = phase_name.to_string();

        let handle = task::spawn(async move {
            let mut requests = 0u64;
            
            while Instant::now() < end && requests < config.max_requests_per_worker {
                let _permit = semaphore.acquire().await;
                
                let scenario_request = config.scenario.generate_request(&config.base_url);
                
                let start_req = Instant::now();
                let response = client
                    .request(scenario_request.method, &scenario_request.url)
                    .headers(scenario_request.headers)
                    .body(scenario_request.body)
                    .send()
                    .await;
                
                let latency = start_req.elapsed();
                
                match response {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        metrics.record_request(
                            &phase_name,
                            &scenario_request.endpoint,
                            status,
                            latency,
                            true,
                        );
                        requests += 1;
                    }
                    Err(e) => {
                        metrics.record_request(
                            &phase_name,
                            &scenario_request.endpoint,
                            0,
                            latency,
                            false,
                        );
                        warn!("Request failed: {}: {}", scenario_request.endpoint, e);
                        requests += 1;
                    }
                }
            }
            
            requests
        });
        
        handles.push(handle);
    }

    // Wait for all workers to complete
    let mut total_requests = 0u64;
    for handle in handles {
        total_requests += handle.await?;
    }

    info!(
        "Phase '{}' completed: {} requests in {:.2}s",
        phase_name,
        total_requests,
        duration.as_secs_f64()
    );

    Ok(())
}

async fn run_distributed_test(
    base_url: &str,
    duration_secs: u64,
    target_rps: u32,
    num_workers: usize,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Distributed test: {} workers, {} RPS, {} seconds",
        num_workers, target_rps, duration_secs
    );
    
    // In a real distributed setup, this would coordinate with remote workers
    // For now, we run multiple local workers
    let config = LoadTestConfig {
        base_url: base_url.to_string(),
        duration: Duration::from_secs(duration_secs),
        target_rps,
        num_workers,
        max_requests_per_worker: u64::MAX,
        warmup: Duration::from_secs(0),
        rampup: Duration::from_secs(0),
        rampdown: Duration::from_secs(0),
        scenario: ScenarioType::Mixed,
    };

    let metrics = Arc::new(LoadTestMetrics::new());
    
    if json_output {
        metrics.set_json_output(true);
    }

    run_load_test(
        base_url,
        duration_secs,
        target_rps,
        num_workers,
        u64::MAX,
        "mixed",
        0,
        0,
        0,
        json_output,
    )
    .await?;

    Ok(())
}

async fn run_endurance_test(
    base_url: &str,
    duration_secs: u64,
    target_rps: u32,
    num_workers: usize,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Endurance test: {} RPS, {} workers, {} seconds ({} hours)",
        target_rps,
        num_workers,
        duration_secs,
        duration_secs / 3600
    );

    run_load_test(
        base_url,
        duration_secs,
        target_rps,
        num_workers,
        u64::MAX,
        "mixed",
        0,
        0,
        0,
        json_output,
    )
    .await?;

    Ok(())
}

async fn run_stress_test(
    base_url: &str,
    start_rate: u32,
    max_rate: u32,
    increment: u32,
    step_duration_secs: u64,
    num_workers: usize,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Stress test: Start {} RPS, Max {} RPS, Increment {} RPS, Step {}s",
        start_rate, max_rate, increment, step_duration_secs
    );

    let metrics = Arc::new(LoadTestMetrics::new());
    
    if json_output {
        metrics.set_json_output(true);
    }

    let mut current_rate = start_rate;
    let mut step = 0;
    
    while current_rate <= max_rate {
        info!("Step {}: Testing at {} RPS", step, current_rate);
        
        run_load_test(
            base_url,
            step_duration_secs,
            current_rate,
            num_workers,
            u64::MAX,
            "mixed",
            0,
            0,
            0,
            json_output,
        )
        .await?;

        // Check if system is failing
        let summary = metrics.summary();
        let error_rate = summary.total_errors as f64 / summary.total_requests as f64 * 100.0;
        
        if error_rate > 10.0 {
            warn!(
                "High error rate ({:.2}%) at {} RPS - system under stress",
                error_rate, current_rate
            );
        }

        current_rate += increment;
        step += 1;
        
        // Reset metrics for next step
        metrics.reset();
    }

    info!("Stress test completed. Maximum tested RPS: {}", current_rate - increment);
    
    Ok(())
}

async fn run_spike_test(
    base_url: &str,
    base_rate: u32,
    spike_rate: u32,
    spike_duration_secs: u64,
    recovery_secs: u64,
    num_spikes: u32,
    num_workers: usize,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Spike test: Base {} RPS, Spike {} RPS, Duration {}s, Recovery {}s, Spikes {}",
        base_rate, spike_rate, spike_duration_secs, recovery_secs, num_spikes
    );

    for spike in 0..num_spikes {
        info!("Spike {}: Starting", spike + 1);
        
        // Base load
        run_load_test(
            base_url,
            10, // Short base period
            base_rate,
            num_workers,
            u64::MAX,
            "mixed",
            0,
            0,
            0,
            json_output,
        )
        .await?;

        // Spike
        info!("Spike {}: Spiking to {} RPS", spike + 1, spike_rate);
        run_load_test(
            base_url,
            spike_duration_secs,
            spike_rate,
            num_workers,
            u64::MAX,
            "mixed",
            0,
            0,
            0,
            json_output,
        )
        .await?;

        // Recovery
        info!("Spike {}: Recovery period", spike + 1);
        run_load_test(
            base_url,
            recovery_secs,
            base_rate,
            num_workers,
            u64::MAX,
            "mixed",
            0,
            0,
            0,
            json_output,
        )
        .await?;
    }

    Ok(())
}

async fn run_soak_test(
    base_url: &str,
    duration_secs: u64,
    target_rps: u32,
    num_workers: usize,
    check_memory: bool,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Soak test: {} RPS, {} workers, {} seconds ({} hours)",
        target_rps,
        num_workers,
        duration_secs,
        duration_secs / 3600
    );

    if check_memory {
        warn!("Memory leak checking is enabled but not implemented in this version");
    }

    run_load_test(
        base_url,
        duration_secs,
        target_rps,
        num_workers,
        u64::MAX,
        "mixed",
        0,
        0,
        0,
        json_output,
    )
    .await?;

    Ok(())
}
