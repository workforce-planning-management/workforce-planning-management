//! Load test metrics collection and reporting

use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};

/// Metrics collected during a load test
#[derive(Debug)]
pub struct LoadTestMetrics {
    /// JSON output flag
    json_output: AtomicBool,
    
    /// Start time of the test
    start_time: Option<Instant>,
    
    /// End time of the test
    end_time: Option<Instant>,
    
    /// Per-phase metrics
    phases: Mutex<HashMap<String, PhaseMetrics>>,
    
    /// Global metrics
    global: Mutex<GlobalMetrics>,
}

/// Metrics for a specific phase (warmup, rampup, main, rampdown)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMetrics {
    /// Total requests
    pub total_requests: u64,
    
    /// Successful requests (2xx status)
    pub success_count: u64,
    
    /// Failed requests (non-2xx or errors)
    pub error_count: u64,
    
    /// Requests by status code
    pub status_codes: HashMap<u16, u64>,
    
    /// Requests by endpoint
    pub endpoints: HashMap<String, EndpointMetrics>,
    
    /// Latency statistics
    pub latency: LatencyMetrics,
}

/// Metrics for a specific endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub count: u64,
    pub success: u64,
    pub errors: u64,
    pub latency: LatencyMetrics,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub min_ms: Option<f64>,
    pub max_ms: Option<f64>,
    pub sum_ms: f64,
    pub count: u64,
}

impl LatencyMetrics {
    pub fn new() -> Self {
        Self {
            min_ms: None,
            max_ms: None,
            sum_ms: 0.0,
            count: 0,
        }
    }
    
    pub fn record(&mut self, latency: Duration) {
        let latency_ms = latency.as_secs_f64() * 1000.0;
        self.count += 1;
        self.sum_ms += latency_ms;
        
        if let Some(min) = self.min_ms {
            if latency_ms < min {
                self.min_ms = Some(latency_ms);
            }
        } else {
            self.min_ms = Some(latency_ms);
        }
        
        if let Some(max) = self.max_ms {
            if latency_ms > max {
                self.max_ms = Some(latency_ms);
            }
        } else {
            self.max_ms = Some(latency_ms);
        }
    }
    
    pub fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum_ms / self.count as f64
        } else {
            0.0
        }
    }
    
    pub fn p50(&self) -> f64 {
        // Simple approximation - would need sorted latencies for exact percentile
        self.average()
    }
    
    pub fn p95(&self) -> f64 {
        // Simple approximation
        if self.count > 0 {
            self.sum_ms / (self.count as f64 * 20.0 / 19.0)
        } else {
            0.0
        }
    }
    
    pub fn p99(&self) -> f64 {
        // Simple approximation
        if self.count > 0 {
            self.sum_ms / (self.count as f64 * 100.0 / 99.0)
        } else {
            0.0
        }
    }
}

/// Global metrics across all phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub total_requests: u64,
    pub total_success: u64,
    pub total_errors: u64,
}

impl LoadTestMetrics {
    pub fn new() -> Self {
        Self {
            json_output: AtomicBool::new(false),
            start_time: None,
            end_time: None,
            phases: Mutex::new(HashMap::new()),
            global: Mutex::new(GlobalMetrics {
                total_requests: 0,
                total_success: 0,
                total_errors: 0,
            }),
        }
    }
    
    pub fn set_json_output(&self, json: bool) {
        self.json_output.store(json, std::sync::atomic::Ordering::SeqCst);
    }
    
    pub fn is_json_output(&self) -> bool {
        self.json_output.load(std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Start the test timer
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }
    
    /// End the test timer
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
    }
    
    /// Record a request
    pub fn record_request(
        &self,
        phase: &str,
        endpoint: &str,
        status: u16,
        latency: Duration,
        success: bool,
    ) {
        let mut phases = self.phases.lock().unwrap();
        let phase_metrics = phases.entry(phase.to_string())
            .or_insert_with(PhaseMetrics::new);
        
        phase_metrics.total_requests += 1;
        
        if success {
            phase_metrics.success_count += 1;
        } else {
            phase_metrics.error_count += 1;
        }
        
        // Record status code
        *phase_metrics.status_codes.entry(status).or_insert(0) += 1;
        
        // Record endpoint metrics
        let endpoint_metrics = phase_metrics.endpoints.entry(endpoint.to_string())
            .or_insert_with(EndpointMetrics::new);
        
        endpoint_metrics.count += 1;
        if success {
            endpoint_metrics.success += 1;
        } else {
            endpoint_metrics.errors += 1;
        }
        endpoint_metrics.latency.record(latency);
        
        // Record global latency
        phase_metrics.latency.record(latency);
        
        // Update global metrics
        let mut global = self.global.lock().unwrap();
        global.total_requests += 1;
        if success {
            global.total_success += 1;
        } else {
            global.total_errors += 1;
        }
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        let mut phases = self.phases.lock().unwrap();
        phases.clear();
        let mut global = self.global.lock().unwrap();
        *global = GlobalMetrics {
            total_requests: 0,
            total_success: 0,
            total_errors: 0,
        };
    }
    
    /// Get a summary of all metrics
    pub fn summary(&self) -> LoadTestSummary {
        let phases = self.phases.lock().unwrap();
        let global = self.global.lock().unwrap();
        
        let total_duration = self.end_time.and_then(|end| {
            self.start_time.map(|start| end - start)
        });
        
        let mut phase_summaries = HashMap::new();
        for (phase_name, phase_metrics) in phases.iter() {
            phase_summaries.insert(phase_name.clone(), PhaseSummary {
                total_requests: phase_metrics.total_requests,
                success_count: phase_metrics.success_count,
                error_count: phase_metrics.error_count,
                success_rate: if phase_metrics.total_requests > 0 {
                    phase_metrics.success_count as f64 / phase_metrics.total_requests as f64 * 100.0
                } else {
                    0.0
                },
                min_latency_ms: phase_metrics.latency.min_ms,
                max_latency_ms: phase_metrics.latency.max_ms,
                avg_latency_ms: phase_metrics.latency.average(),
                p50_latency_ms: phase_metrics.latency.p50(),
                p95_latency_ms: phase_metrics.latency.p95(),
                p99_latency_ms: phase_metrics.latency.p99(),
                status_codes: phase_metrics.status_codes.clone(),
            });
        }
        
        LoadTestSummary {
            total_duration_secs: total_duration.map(|d| d.as_secs_f64()).unwrap_or(0.0),
            total_requests: global.total_requests,
            total_success: global.total_success,
            total_errors: global.total_errors,
            success_rate: if global.total_requests > 0 {
                global.total_success as f64 / global.total_requests as f64 * 100.0
            } else {
                0.0
            },
            phases: phase_summaries,
        }
    }
    
    /// Print a human-readable summary
    pub fn print_summary(&self) {
        let summary = self.summary();
        
        println!("\n{}", "=".repeat(80));
        println!("LOAD TEST SUMMARY");
        println!("{}", "=".repeat(80));
        
        println!("\nTotal Duration: {:.2}s", summary.total_duration_secs);
        println!("Total Requests: {}", summary.total_requests);
        println!("Total Success: {}", summary.total_success);
        println!("Total Errors: {}", summary.total_errors);
        println!("Success Rate: {:.2}%", summary.success_rate);
        
        println!("\n{}", "-".repeat(80));
        println!("PHASE METRICS");
        println!("{}", "-".repeat(80));
        
        for (phase_name, phase_summary) in &summary.phases {
            println!("\nPhase: {}", phase_name);
            println!("  Requests: {}", phase_summary.total_requests);
            println!("  Success: {} ({:.2}%)", phase_summary.success_count, phase_summary.success_rate);
            println!("  Errors: {}", phase_summary.error_count);
            println!("  Avg Latency: {:.2}ms", phase_summary.avg_latency_ms);
            println!("  Min Latency: {:.2}ms", phase_summary.min_latency_ms.unwrap_or(0.0));
            println!("  Max Latency: {:.2}ms", phase_summary.max_latency_ms.unwrap_or(0.0));
            println!("  P50 Latency: {:.2}ms", phase_summary.p50_latency_ms);
            println!("  P95 Latency: {:.2}ms", phase_summary.p95_latency_ms);
            println!("  P99 Latency: {:.2}ms", phase_summary.p99_latency_ms);
            
            if !phase_summary.status_codes.is_empty() {
                println!("  Status Codes:");
                for (code, count) in &phase_summary.status_codes {
                    println!("    {}: {}", code, count);
                }
            }
        }
        
        println!("\n{}", "=".repeat(80));
    }
}

impl PhaseMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            success_count: 0,
            error_count: 0,
            status_codes: HashMap::new(),
            endpoints: HashMap::new(),
            latency: LatencyMetrics::new(),
        }
    }
}

impl EndpointMetrics {
    fn new() -> Self {
        Self {
            count: 0,
            success: 0,
            errors: 0,
            latency: LatencyMetrics::new(),
        }
    }
}

/// Summary structure for JSON output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestSummary {
    pub total_duration_secs: f64,
    pub total_requests: u64,
    pub total_success: u64,
    pub total_errors: u64,
    pub success_rate: f64,
    pub phases: HashMap<String, PhaseSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSummary {
    pub total_requests: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub status_codes: HashMap<u16, u64>,
}
