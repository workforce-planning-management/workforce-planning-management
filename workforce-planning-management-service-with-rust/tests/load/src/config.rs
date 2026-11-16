//! Load test configuration

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::scenarios::ScenarioType;

/// Configuration for a load test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    /// Base URL of the WPM service (e.g., http://localhost:1241)
    pub base_url: String,
    
    /// Total duration of the test
    pub duration: Duration,
    
    /// Target requests per second
    pub target_rps: u32,
    
    /// Number of concurrent workers
    pub num_workers: usize,
    
    /// Maximum requests per worker (0 = unlimited)
    pub max_requests_per_worker: u64,
    
    /// Warmup period before main test
    pub warmup: Duration,
    
    /// Rampup period to reach target RPS
    pub rampup: Duration,
    
    /// Rampdown period after main test
    pub rampdown: Duration,
    
    /// Test scenario to run
    pub scenario: ScenarioType,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:1241".to_string(),
            duration: Duration::from_secs(60),
            target_rps: 100,
            num_workers: 4,
            max_requests_per_worker: 10000,
            warmup: Duration::from_secs(10),
            rampup: Duration::from_secs(20),
            rampdown: Duration::from_secs(10),
            scenario: ScenarioType::Mixed,
        }
    }
}

impl LoadTestConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        use std::env;
        
        Self {
            base_url: env::var("WPM_BASE_URL").unwrap_or_else(|_| "http://localhost:1241".to_string()),
            duration: Duration::from_secs(
                env::var("WPM_LOAD_TEST_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
            target_rps: env::var("WPM_TARGET_RPS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            num_workers: env::var("WPM_LOAD_TEST_WORKERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(4),
            max_requests_per_worker: env::var("WPM_MAX_REQUESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
            warmup: Duration::from_secs(
                env::var("WPM_WARMUP_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            ),
            rampup: Duration::from_secs(
                env::var("WPM_RAMPUP_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20),
            ),
            rampdown: Duration::from_secs(
                env::var("WPM_RAMPDOWN_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            ),
            scenario: env::var("WPM_SCENARIO")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(ScenarioType::Mixed),
        }
    }
}
