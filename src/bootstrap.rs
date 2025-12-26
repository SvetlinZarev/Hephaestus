use crate::config::Collectors;
use crate::metrics::{Collector, Metric};
use crate::{data_source, metrics};
use prometheus::Registry;
use std::sync::{Arc, Mutex};
use sysinfo::System;

pub fn init_collectors(
    config: &Collectors,
    registry: &Registry,
) -> anyhow::Result<Vec<Box<dyn Collector>>> {
    let mut collectors = vec![];
    let system = Arc::new(Mutex::new(System::new_all()));

    let data_source = data_source::cpu_usage::CpuUsage::new(system.clone());
    let metric = metrics::cpu_usage::CpuUsage::new(config.cpu_usage.clone(), data_source)?;
    if metric.enabled() {
        let collector = metric.register(registry)?;
        collectors.push(collector);
    }

    let data_source = data_source::cpu_frequency::CpuFrequency::new(system.clone());
    let metric =
        metrics::cpu_frequency::CpuFrequency::new(config.cpu_frequency.clone(), data_source)?;
    if metric.enabled() {
        let collector = metric.register(registry)?;
        collectors.push(collector);
    }

    let data_source = data_source::memory_usage::MemoryUsage::new(system.clone());
    let metric = metrics::memory_usage::MemoryUsage::new(config.memory_usage.clone(), data_source)?;
    if metric.enabled() {
        let collector = metric.register(registry)?;
        collectors.push(collector);
    }

    Ok(collectors)
}
