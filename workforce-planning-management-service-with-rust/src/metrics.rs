//! Prometheus metrics for the WPM service.
//!
//! A process-wide [`Registry`](prometheus::Registry) with lifecycle
//! counters plus live workforce **gauges** (headcount, open
//! requisitions, pending leave), rendered at `GET /metrics.prom`
//! (`metrics_path: /metrics.prom`).

use std::sync::OnceLock;

use prometheus::{Counter, Encoder, IntGauge, Opts, Registry, TextEncoder};

/// Content type for the Prometheus text-exposition format.
pub const CONTENT_TYPE: &str = "text/plain; version=0.0.4";

/// Process-wide registry and the WPM metric handles.
pub struct Metrics {
    /// The underlying registry.
    pub registry: Registry,
    /// Workers hired (application `hired` transitions).
    pub worker_hired_total: Counter,
    /// Workers activated (`onboarding → active`).
    pub worker_activated_total: Counter,
    /// Workers terminated.
    pub worker_terminated_total: Counter,
    /// Leave requests decided (approved + rejected).
    pub leave_decided_total: Counter,
    /// Payroll runs calculated.
    pub payroll_calculated_total: Counter,
    /// Live gauge: active workers.
    pub workers_active: IntGauge,
    /// Live gauge: open requisitions.
    pub requisitions_open: IntGauge,
    /// Live gauge: leave requests awaiting decision.
    pub leave_pending: IntGauge,
}

impl Metrics {
    /// Construct the metric set into a fresh registry. The `.expect`s
    /// are infallible: static opts, empty registry.
    fn new() -> Self {
        let registry = Registry::new();
        let counter = |name: &str, help: &str| {
            Counter::with_opts(Opts::new(name, help)).expect("static counter opts are always valid")
        };
        let gauge = |name: &str, help: &str| {
            IntGauge::with_opts(Opts::new(name, help)).expect("static gauge opts are always valid")
        };
        let worker_hired_total = counter("worker_hired_total", "Total workers hired.");
        let worker_activated_total =
            counter("worker_activated_total", "Total workers activated.");
        let worker_terminated_total =
            counter("worker_terminated_total", "Total workers terminated.");
        let leave_decided_total = counter("leave_decided_total", "Total leave requests decided.");
        let payroll_calculated_total =
            counter("payroll_calculated_total", "Total payroll runs calculated.");
        let workers_active = gauge("workers_active", "Workers currently active.");
        let requisitions_open = gauge("requisitions_open", "Requisitions currently open.");
        let leave_pending = gauge("leave_pending", "Leave requests awaiting decision.");
        for collector in [
            Box::new(worker_hired_total.clone()) as Box<dyn prometheus::core::Collector>,
            Box::new(worker_activated_total.clone()),
            Box::new(worker_terminated_total.clone()),
            Box::new(leave_decided_total.clone()),
            Box::new(payroll_calculated_total.clone()),
            Box::new(workers_active.clone()),
            Box::new(requisitions_open.clone()),
            Box::new(leave_pending.clone()),
        ] {
            registry
                .register(collector)
                .expect("registering a freshly-constructed collector cannot fail");
        }
        Self {
            registry,
            worker_hired_total,
            worker_activated_total,
            worker_terminated_total,
            leave_decided_total,
            payroll_calculated_total,
            workers_active,
            requisitions_open,
            leave_pending,
        }
    }

    /// The process-wide metrics, initialised on first access.
    #[must_use]
    pub fn global() -> &'static Self {
        static METRICS: OnceLock<Metrics> = OnceLock::new();
        METRICS.get_or_init(Self::new)
    }

    /// Render the registry in Prometheus text-exposition format.
    ///
    /// # Panics
    ///
    /// Not in practice: the text encoder writes into a `Vec` and
    /// produces valid UTF-8.
    #[must_use]
    pub fn render(&self) -> String {
        let encoder = TextEncoder::new();
        let mut buf = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buf)
            .expect("text encoder writes to a Vec which never fails");
        String::from_utf8(buf).expect("Prometheus text encoder produces UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counters and gauges register and render in valid exposition
    /// format.
    #[test]
    fn render_yields_valid_prometheus_text() {
        let m = Metrics::global();
        m.worker_hired_total.inc();
        m.workers_active.set(42);
        let body = m.render();
        assert!(body.contains("# TYPE worker_hired_total counter"));
        assert!(body.contains("# TYPE workers_active gauge"));
        assert!(body.contains("workers_active 42"));
    }
}
