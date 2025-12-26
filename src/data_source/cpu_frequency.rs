use anyhow::anyhow;
use std::sync::{Arc, Mutex};
use sysinfo::System;

pub trait DataSource {
    fn cpu_frequency(&self) -> anyhow::Result<Vec<u64>>;
}

#[derive(Clone)]
pub struct CpuFrequency {
    system: Arc<Mutex<System>>,
}

impl CpuFrequency {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self { system }
    }
}

impl DataSource for CpuFrequency {
    fn cpu_frequency(&self) -> anyhow::Result<Vec<u64>> {
        let mut system = self.system.lock().map_err(|e| {
            anyhow!(
                "Failed to refresh CPU frequency statistics due to poisoned mutex: {}",
                e
            )
        })?;

        system.refresh_cpu_frequency();
        Ok(system.cpus().iter().map(|cpu| cpu.frequency()).collect())
    }
}
