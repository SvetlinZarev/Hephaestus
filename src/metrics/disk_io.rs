use crate::domain::{Collector, Metric};
use crate::metrics::no_operation::NoOpCollector;
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

#[derive(Debug, Clone)]
pub struct DeviceIoStats {
    pub device_name: String,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

#[derive(Debug, Clone)]
pub struct DiskIoStats {
    pub devices: Vec<DeviceIoStats>,
}

pub trait DataSource {
    fn disk_io(&self) -> impl Future<Output = anyhow::Result<DiskIoStats>> + Send;
}

#[derive(Clone)]
struct Metrics {
    bytes_read: IntGaugeVec,
    bytes_written: IntGaugeVec,
}

impl Metrics {
    fn register(registry: &Registry) -> anyhow::Result<Self> {
        let read_opts = Opts::new(
            "system_disk_bytes_read_total",
            "Total bytes read from disk (cumulative)",
        );
        let bytes_read = IntGaugeVec::new(read_opts, &["device"])?;
        registry.register(Box::new(bytes_read.clone()))?;

        let write_opts = Opts::new(
            "system_disk_bytes_written_total",
            "Total bytes written to disk (cumulative)",
        );
        let bytes_written = IntGaugeVec::new(write_opts, &["device"])?;
        registry.register(Box::new(bytes_written.clone()))?;

        Ok(Self {
            bytes_read,
            bytes_written,
        })
    }
}

pub struct DiskIo {
    config: Config,
}

impl DiskIo {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl<T> Metric<T> for DiskIo
where
    T: DataSource + Send + Sync + 'static,
{
    fn register(self, registry: &Registry, data_source: T) -> anyhow::Result<Box<dyn Collector>> {
        if !self.config.enabled {
            return Ok(Box::new(NoOpCollector::new()));
        }

        let metrics = Metrics::register(registry)?;
        Ok(Box::new(DiskIoCollector::new(metrics, data_source)))
    }
}

struct DiskIoCollector<T> {
    metrics: Metrics,
    data_source: T,
}

impl<T> DiskIoCollector<T> {
    fn new(metrics: Metrics, data_source: T) -> Self {
        Self {
            metrics,
            data_source,
        }
    }

    fn should_collect(&self, device_name: &str) -> bool {
        !(device_name.starts_with("loop") || device_name.starts_with("zram"))
    }
}

#[async_trait::async_trait]
impl<T> Collector for DiskIoCollector<T>
where
    T: DataSource + Send + Sync + 'static,
{
    async fn collect(&self) -> anyhow::Result<()> {
        let stats = self.data_source.disk_io().await?;

        for dev in stats.devices {
            if self.should_collect(&dev.device_name) {
                let label = &[dev.device_name.as_str()];

                self.metrics
                    .bytes_read
                    .with_label_values(label)
                    .set(dev.bytes_read as i64);

                self.metrics
                    .bytes_written
                    .with_label_values(label)
                    .set(dev.bytes_written as i64);
            }
        }

        Ok(())
    }
}
