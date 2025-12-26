use prometheus::Registry;

pub mod cpu_frequency;
pub mod cpu_usage;
pub mod memory_usage;

pub trait Metric {
    fn name(&self) -> &'static str;

    fn enabled(&self) -> bool;

    fn register(&self, registry: &Registry) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait Collector: Send + Sync {
    async fn collect(&self) -> anyhow::Result<()>;
}
