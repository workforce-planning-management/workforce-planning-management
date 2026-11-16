//! Load test scenarios
//!
//! Different scenarios simulate different usage patterns of the WPM service.

use std::collections::HashMap;
use std::fmt;

use rand::prelude::SliceRandom;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

/// Type of load test scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScenarioType {
    /// Mixed workload - simulates typical usage
    Mixed,
    /// Read-heavy - mostly GET requests
    ReadHeavy,
    /// Write-heavy - mostly POST/PUT requests
    WriteHeavy,
    /// Auth-heavy - focuses on authentication endpoints
    AuthHeavy,
    /// API-heavy - focuses on API endpoints
    ApiHeavy,
    /// Stress - maximum load
    Stress,
    /// Custom - user-defined
    Custom,
}

impl fmt::Display for ScenarioType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioType::Mixed => write!(f, "mixed"),
            ScenarioType::ReadHeavy => write!(f, "read-heavy"),
            ScenarioType::WriteHeavy => write!(f, "write-heavy"),
            ScenarioType::AuthHeavy => write!(f, "auth-heavy"),
            ScenarioType::ApiHeavy => write!(f, "api-heavy"),
            ScenarioType::Stress => write!(f, "stress"),
            ScenarioType::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for ScenarioType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mixed" => Ok(ScenarioType::Mixed),
            "read-heavy" | "read_heavy" | "read" => Ok(ScenarioType::ReadHeavy),
            "write-heavy" | "write_heavy" | "write" => Ok(ScenarioType::WriteHeavy),
            "auth-heavy" | "auth_heavy" | "auth" => Ok(ScenarioType::AuthHeavy),
            "api-heavy" | "api_heavy" | "api" => Ok(ScenarioType::ApiHeavy),
            "stress" => Ok(ScenarioType::Stress),
            "custom" => Ok(ScenarioType::Custom),
            _ => Err(format!("Unknown scenario type: {}", s)),
        }
    }
}

/// A request to be made during load testing
#[derive(Debug, Clone)]
pub struct Request {
    pub method: reqwest::Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: String,
    pub endpoint: String,
}

impl Request {
    pub fn new(method: reqwest::Method, endpoint: &str, base_url: &str) -> Self {
        Self {
            method,
            url: format!("{}/{}", base_url.trim_end_matches('/'), endpoint.trim_start_matches('/')),
            headers: HeaderMap::new(),
            body: String::new(),
            endpoint: endpoint.to_string(),
        }
    }
    
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), HeaderValue::from_str(value)) {
            self.headers.insert(name, val);
        }
        self
    }
    
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
    
    pub fn with_json_body(mut self, json: serde_json::Value) -> Self {
        self.body = json.to_string();
        self.with_header("Content-Type", "application/json")
    }
}

/// Trait for generating requests
pub trait RequestGenerator {
    fn generate_request(&self, base_url: &str) -> Request;
}

/// Scenario that generates mixed workload
pub struct MixedScenario;

impl RequestGenerator for MixedScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        // Weighted random selection of different request types
        let roll = rand::random::<f64>() * 100.0;
        
        if roll < 60.0 {
            // 60% - Read operations
            self.generate_read_request(base_url)
        } else if roll < 80.0 {
            // 20% - Write operations
            self.generate_write_request(base_url)
        } else if roll < 90.0 {
            // 10% - Auth operations
            self.generate_auth_request(base_url)
        } else {
            // 10% - Health/metrics
            self.generate_health_request(base_url)
        }
    }
}

impl MixedScenario {
    fn generate_read_request(&self, base_url: &str) -> Request {
        let endpoints = [
            "api/workers?limit=10",
            "api/workers",
            "api/org-chart",
            "api/departments",
            "api/teams",
            "api/rotas",
            "api/shifts",
            "api/leave-requests",
            "api/time-entries",
        ];
        
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
    }
    
    fn generate_write_request(&self, base_url: &str) -> Request {
        let endpoints = [
            ("POST", "api/time-entries"),
            ("POST", "api/leave-requests"),
            ("PUT", "api/workers/{pid}"),
        ];
        
        let (method, endpoint) = endpoints.choose(&mut rand::thread_rng()).unwrap();
        let method: reqwest::Method = method.parse().unwrap();
        
        Request::new(method, endpoint, base_url)
            .with_header("Content-Type", "application/json")
            .with_body(r#"{"test": "data"}"#)
    }
    
    fn generate_auth_request(&self, base_url: &str) -> Request {
        Request::new(reqwest::Method::GET, "api/auth/self", base_url)
            .with_header("Authorization", "Bearer test-token")
    }
    
    fn generate_health_request(&self, base_url: &str) -> Request {
        let endpoints = ["_health/live", "_health/ready", "_ping"];
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
    }
}

/// Scenario that generates read-heavy workload
pub struct ReadHeavyScenario;

impl RequestGenerator for ReadHeavyScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        let endpoints = [
            "api/workers?limit=10",
            "api/workers",
            "api/org-chart",
            "api/departments",
            "api/teams",
            "api/rotas",
            "api/shifts",
            "api/leave-requests",
            "api/time-entries",
            "api/announcements",
            "api/skills",
            "api/reviews",
        ];
        
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
    }
}

/// Scenario that generates write-heavy workload
pub struct WriteHeavyScenario;

impl RequestGenerator for WriteHeavyScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        let endpoints = [
            ("POST", "api/time-entries"),
            ("POST", "api/leave-requests"),
            ("PUT", "api/workers/{pid}"),
            ("POST", "api/shifts"),
            ("POST", "api/announcements"),
        ];
        
        let (method, endpoint) = endpoints.choose(&mut rand::thread_rng()).unwrap();
        let method: reqwest::Method = method.parse().unwrap();
        
        Request::new(method, endpoint, base_url)
            .with_header("Content-Type", "application/json")
            .with_body(&self.generate_body(endpoint))
    }
}

impl WriteHeavyScenario {
    fn generate_body(&self, endpoint: &str) -> String {
        match endpoint {
            "api/time-entries" => r#"{"worker_pid": "worker:123", "date": "2026-01-01", "hours": 8.0, "kind": "regular"}"#.to_string(),
            "api/leave-requests" => r#"{"worker_pid": "worker:123", "kind": "annual", "start_date": "2026-01-01", "end_date": "2026-01-05", "reason": "Vacation"}"#.to_string(),
            "api/shifts" => r#"{"worker_pid": "worker:123", "start_time": "2026-01-01T09:00:00Z", "end_time": "2026-01-01T17:00:00Z"}"#.to_string(),
            "api/announcements" => r#"{"title": "Test Announcement", "content": "This is a test", "audience": "all"}"#.to_string(),
            _ => r#"{"test": "data"}"#.to_string(),
        }
    }
}

/// Scenario that focuses on authentication
pub struct AuthHeavyScenario;

impl RequestGenerator for AuthHeavyScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        let endpoints = [
            "api/auth/self",
            "api/auth/refresh",
            "api/workers/me",
        ];
        
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
            .with_header("Authorization", "Bearer test-token")
    }
}

/// Scenario that focuses on API endpoints
pub struct ApiHeavyScenario;

impl RequestGenerator for ApiHeavyScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        let endpoints = [
            "api/workers?limit=5",
            "api/org-chart",
            "api/metrics",
            "api/insights",
            "api/workforce/working-time",
            "api/skill-gaps",
            "api/capacity",
        ];
        
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
    }
}

/// Scenario for stress testing
pub struct StressScenario;

impl RequestGenerator for StressScenario {
    fn generate_request(&self, base_url: &str) -> Request {
        // Stress scenario uses heavier requests
        let endpoints = [
            "api/workers?limit=100",
            "api/org-chart?department=all",
            "api/workers?with_related=true",
        ];
        
        let endpoint = endpoints.choose(&mut rand::thread_rng()).unwrap();
        Request::new(reqwest::Method::GET, endpoint, base_url)
    }
}

/// Main scenario type that delegates to specific scenarios
pub struct Scenario {
    scenario_type: ScenarioType,
}

impl Scenario {
    pub fn new(scenario_type: ScenarioType) -> Self {
        Self { scenario_type }
    }
}

impl RequestGenerator for Scenario {
    fn generate_request(&self, base_url: &str) -> Request {
        match self.scenario_type {
            ScenarioType::Mixed => MixedScenario.generate_request(base_url),
            ScenarioType::ReadHeavy => ReadHeavyScenario.generate_request(base_url),
            ScenarioType::WriteHeavy => WriteHeavyScenario.generate_request(base_url),
            ScenarioType::AuthHeavy => AuthHeavyScenario.generate_request(base_url),
            ScenarioType::ApiHeavy => ApiHeavyScenario.generate_request(base_url),
            ScenarioType::Stress => StressScenario.generate_request(base_url),
            ScenarioType::Custom => MixedScenario.generate_request(base_url),
        }
    }
}

impl ScenarioType {
    pub fn generator(&self) -> Box<dyn RequestGenerator> {
        match self {
            ScenarioType::Mixed => Box::new(MixedScenario),
            ScenarioType::ReadHeavy => Box::new(ReadHeavyScenario),
            ScenarioType::WriteHeavy => Box::new(WriteHeavyScenario),
            ScenarioType::AuthHeavy => Box::new(AuthHeavyScenario),
            ScenarioType::ApiHeavy => Box::new(ApiHeavyScenario),
            ScenarioType::Stress => Box::new(StressScenario),
            ScenarioType::Custom => Box::new(MixedScenario),
        }
    }
    
    pub fn generate_request(&self, base_url: &str) -> Request {
        self.generator().generate_request(base_url)
    }
}
