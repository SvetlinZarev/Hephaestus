use crate::collector::{Collector, Metric};
use crate::data_source::cpu_usage::DataSource;
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

pub struct CpuUsage<T> {
    config: Config,
    core_usage: GaugeVec,
    total_usage: Gauge,
    data_source: T,
}

impl<T> CpuUsage<T>
where
    T: DataSource,
{
    pub fn new(config: Config, data_source: T) -> anyhow::Result<Self> {
        let opts = Opts::new("system_cpu_core_usage", "CPU usage percentage per core");
        let core_usage = GaugeVec::new(opts, &["core"])?;
        let total_usage = Gauge::new("system_cpu_total_usage", "CPU usage across all cores")?;

        Ok(Self {
            config,
            core_usage,
            total_usage,
            data_source,
        })
    }
}

impl<T> Metric for CpuUsage<T>
where
    T: DataSource 
{
    fn name(&self) -> &'static str {
        "cpu-usage"
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.core_usage.clone()))?;
        registry.register(Box::new(self.total_usage.clone()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T> Collector for CpuUsage<T>
where
    T: DataSource + Send + Sync + Clone + 'static,
{
    async fn collect(&self) -> anyhow::Result<()> {
        let datasource = self.data_source.clone();
        let total_usage = self.total_usage.clone();
        let core_usage = self.core_usage.clone();

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let (overall, per_core) = datasource.cpu_usage()?;
            total_usage.set(overall);

            for (idx, &usage) in per_core.iter().enumerate() {
                core_usage
                    .with_label_values(&[format!("{}", idx)])
                    .set(usage);
            }

            Ok(())
        })
        .await??;

        Ok(())
    }
}
