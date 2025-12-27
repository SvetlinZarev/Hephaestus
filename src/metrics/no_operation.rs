use crate::domain::Collector;

pub struct NoOpCollector {
    //
}

impl NoOpCollector {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Collector for NoOpCollector {
    async fn collect(&self) -> anyhow::Result<()> {
        // do nothing by design
        Ok(())
    }
}
