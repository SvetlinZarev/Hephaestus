use anyhow::anyhow;
use std::sync::{Arc, Mutex};
use sysinfo::{MemoryRefreshKind, System};

pub trait DataSource {
    fn swap(&self) -> anyhow::Result<(u64, u64, u64)>;
    fn ram(&self) -> anyhow::Result<(u64, u64, u64, u64)>;
}

#[derive(Clone)]
pub struct MemoryUsage {
    system: Arc<Mutex<System>>,
}

impl MemoryUsage {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self { system }
    }
}

impl DataSource for MemoryUsage {
    fn swap(&self) -> anyhow::Result<(u64, u64, u64)> {
        let mut system = self.system.lock().map_err(|e| {
            anyhow!(
                "Failed to refresh SWAP usage statistics due to poisoned mutex: {}",
                e
            )
        })?;

        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
        Ok((system.total_swap(), system.used_swap(), system.free_swap()))
    }

    fn ram(&self) -> anyhow::Result<(u64, u64, u64, u64)> {
        let mut system = self.system.lock().map_err(|e| {
            anyhow!(
                "Failed to refresh RAM usage statistics due to poisoned mutex: {}",
                e
            )
        })?;

        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        Ok((
            system.total_memory(),
            system.used_memory(),
            system.free_memory(),
            system.available_memory(),
        ))
    }
}
