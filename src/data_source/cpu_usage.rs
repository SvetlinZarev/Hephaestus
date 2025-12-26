use anyhow::anyhow;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use sysinfo::System;

pub trait DataSource {
    fn cpu_usage(&self) -> anyhow::Result<(f64, Vec<f64>)>;
}

struct Inner {
    system: Arc<Mutex<System>>,
    last_refresh: Option<Instant>,
}

impl Inner {
    fn new(system: Arc<Mutex<System>>) -> Self {
        Self {
            system,
            last_refresh: None,
        }
    }

    fn needs_refresh(&self) -> bool {
        self.last_refresh
            .map(|t| t.elapsed() > sysinfo::MINIMUM_CPU_UPDATE_INTERVAL)
            .unwrap_or(true)
    }

    fn stats(&mut self) -> anyhow::Result<(f64, Vec<f64>)> {
        let mut system = self.system.lock().map_err(|e| {
            anyhow!(
                "Failed to refresh CPU usage statistics due to poisoned mutex: {}",
                e
            )
        })?;

        if self.needs_refresh() {
            system.refresh_cpu_usage();
            self.last_refresh = Some(Instant::now());
        }

        let overall_usage = system.global_cpu_usage() as f64;
        let core_usage = system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .collect();

        Ok((overall_usage, core_usage))
    }
}

#[derive(Clone)]
pub struct CpuUsage {
    inner: Arc<Mutex<Inner>>,
}

impl CpuUsage {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new(system))),
        }
    }
}

impl DataSource for CpuUsage {
    fn cpu_usage(&self) -> anyhow::Result<(f64, Vec<f64>)> {
        self.inner
            .lock()
            .map_err(|e| {
                anyhow!(
                    "Failed to refresh CPU usage statistics due to poisoned mutex: {}",
                    e
                )
            })?
            .stats()
    }
}
