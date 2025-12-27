use crate::domain::{Collector, Metric};
use crate::metrics::no_operation::NoOpCollector;
use prometheus::{IntGaugeVec, Opts, Registry};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub watch_interfaces: Option<Vec<String>>,
    pub ignore_interfaces: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_interfaces: None,
            ignore_interfaces: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceStats {
    pub interface: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

pub struct NetworkIoStats {
    pub interfaces: Vec<InterfaceStats>,
}

pub trait DataSource {
    fn network_io(&self) -> impl Future<Output = anyhow::Result<NetworkIoStats>> + Send;
}

#[derive(Clone)]
struct Metrics {
    bytes_sent: IntGaugeVec,
    bytes_received: IntGaugeVec,

    packets_sent: IntGaugeVec,
    packets_received: IntGaugeVec,
}

impl Metrics {
    fn register(registry: &Registry) -> anyhow::Result<Self> {
        let sent_opts = Opts::new(
            "system_network_transmit_bytes_total",
            "Total bytes sent (cumulative)",
        );
        let bytes_sent = IntGaugeVec::new(sent_opts, &["device"])?;
        registry.register(Box::new(bytes_sent.clone()))?;

        let recv_opts = Opts::new(
            "system_network_receive_bytes_total",
            "Total bytes received (cumulative)",
        );
        let bytes_received = IntGaugeVec::new(recv_opts, &["device"])?;
        registry.register(Box::new(bytes_received.clone()))?;

        let sent_opts = Opts::new(
            "system_network_transmit_packets_total",
            "Total packets sent (cumulative)",
        );
        let packets_sent = IntGaugeVec::new(sent_opts, &["device"])?;
        registry.register(Box::new(packets_sent.clone()))?;

        let recv_opts = Opts::new(
            "system_network_receive_packets_total",
            "Total packets received (cumulative)",
        );
        let packets_received = IntGaugeVec::new(recv_opts, &["device"])?;
        registry.register(Box::new(packets_received.clone()))?;

        Ok(Self {
            bytes_sent,
            bytes_received,
            packets_sent,
            packets_received,
        })
    }
}

pub struct NetworkIo {
    config: Config,
}
impl NetworkIo {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl<T> Metric<T> for NetworkIo
where
    T: DataSource + Send + Sync + 'static,
{
    fn register(self, registry: &Registry, data_source: T) -> anyhow::Result<Box<dyn Collector>> {
        if !self.config.enabled {
            return Ok(Box::new(NoOpCollector::new()));
        }

        let metrics = Metrics::register(registry)?;
        Ok(Box::new(NetworkIoCollector::new(
            self.config,
            metrics,
            data_source,
        )))
    }
}

struct NetworkIoCollector<T> {
    config: Config,
    metrics: Metrics,
    data_source: T,
}

impl<T> NetworkIoCollector<T> {
    fn new(config: Config, metrics: Metrics, data_source: T) -> Self {
        Self {
            config,
            metrics,
            data_source,
        }
    }

    fn should_collect(&self, interface_name: &str) -> bool {
        if let Some(watch) = &self.config.watch_interfaces {
            return watch.iter().any(|i| i == interface_name);
        }

        if let Some(ignore) = &self.config.ignore_interfaces {
            return !ignore.iter().any(|i| i == interface_name);
        }

        true
    }
}

#[async_trait::async_trait]
impl<T> Collector for NetworkIoCollector<T>
where
    T: DataSource + Send + Sync + 'static,
{
    async fn collect(&self) -> anyhow::Result<()> {
        let stats = self.data_source.network_io().await?;

        for iface in stats.interfaces {
            if self.should_collect(&iface.interface) {
                let label = &[iface.interface.as_str()];

                self.metrics
                    .bytes_sent
                    .with_label_values(label)
                    .set(iface.bytes_sent as i64);

                self.metrics
                    .bytes_received
                    .with_label_values(label)
                    .set(iface.bytes_received as i64);

                self.metrics
                    .packets_sent
                    .with_label_values(label)
                    .set(iface.packets_sent as i64);

                self.metrics
                    .packets_received
                    .with_label_values(label)
                    .set(iface.packets_received as i64);
            }
        }

        Ok(())
    }
}
