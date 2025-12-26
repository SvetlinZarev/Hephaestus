use crate::collector::{Collector, Metric};
use crate::data_source::cpu_frequency::DataSource;
use prometheus::{IntGaugeVec, Opts, Registry};
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

pub struct CpuFrequency<T> {
    config: Config,
    core_freq: IntGaugeVec,
    data_source: T,
}

impl<T> CpuFrequency<T>
where
    T: DataSource,
{
    pub fn new(config: Config, data_source: T) -> anyhow::Result<Self> {
        let opts = Opts::new("system_cpu_core_frequency", "CPU core frequency");
        let core_freq = IntGaugeVec::new(opts, &["core"])?;

        Ok(Self {
            config,
            core_freq,
            data_source,
        })
    }
}

impl<T> Metric for CpuFrequency<T>
where
    T: DataSource,
{
    fn name(&self) -> &'static str {
        "cpu-frequency"
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.core_freq.clone()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T> Collector for CpuFrequency<T>
where
    T: DataSource + Send + Sync + Clone + 'static,
{
    async fn collect(&self) -> anyhow::Result<()> {
        let data_source = self.data_source.clone();
        let core_freq = self.core_freq.clone();

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let freq = data_source.cpu_frequency()?;
            for (idx, freq) in freq.into_iter().enumerate() {
                core_freq
                    .with_label_values(&[format!("{}", idx)])
                    .set(freq as i64);
            }

            Ok(())
        })
        .await??;

        Ok(())
    }
}
