use crate::domain::{Collector, Metric};

use crate::metrics::no_operation::NoOpCollector;
use prometheus::{Gauge, GaugeVec, Opts, Registry};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub trait DataSource {
    fn cpu_usage(&self) -> impl Future<Output = anyhow::Result<CpuUsageStats>> + Send;
}

#[derive(Debug, Clone)]
pub struct CpuUsageStats {
    pub total: f64,
    pub cores: Vec<f64>,
}

pub struct CpuUsage {
    config: Config,
}

impl CpuUsage {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl<T> Metric<T> for CpuUsage
where
    T: DataSource + Send + Sync + 'static,
{
    fn register(self, registry: &Registry, data_source: T) -> anyhow::Result<Box<dyn Collector>> {
        if !self.config.enabled {
            return Ok(Box::new(NoOpCollector::new()));
        }

        let metrics = Metrics::register(registry)?;
        Ok(Box::new(CpuUsageCollector::new(metrics, data_source)))
    }
}

#[derive(Clone)]
struct Metrics {
    total_usage: Gauge,
    core_usage: GaugeVec,
}

impl Metrics {
    fn register(registry: &Registry) -> anyhow::Result<Self> {
        let total_usage = Gauge::new(
            "system_cpu_usage_ratio",
            "Overall CPU usage as a ratio (0.0 to 1.0)",
        )?;
        registry.register(Box::new(total_usage.clone()))?;

        let core_usage_opts = Opts::new(
            "system_cpu_core_usage_ratio",
            "Per-core CPU usage as a ratio (0.0 to 1.0)",
        );
        let core_usage = GaugeVec::new(core_usage_opts, &["core"])?;
        registry.register(Box::new(core_usage.clone()))?;

        Ok(Self {
            total_usage,
            core_usage,
        })
    }
}

struct CpuUsageCollector<T> {
    metrics: Metrics,
    data_source: T,
}

impl<T> CpuUsageCollector<T>
where
    T: DataSource + Send + Sync + 'static,
{
    fn new(metrics: Metrics, data_source: T) -> Self {
        Self {
            metrics,
            data_source,
        }
    }
}

#[async_trait::async_trait]
impl<T> Collector for CpuUsageCollector<T>
where
    T: DataSource + Send + Sync + 'static,
{
    async fn collect(&self) -> anyhow::Result<()> {
        let stats = self.data_source.cpu_usage().await?;

        self.metrics.total_usage.set(stats.total);
        for (core, &usage) in stats.cores.iter().enumerate() {
            self.metrics
                .core_usage
                .with_label_values(&[format!("{}", core)])
                .set(usage);
        }

        Ok(())
    }
}
