use crate::config::Collectors;
use crate::domain::Collector;
use prometheus::Registry;

pub fn init_collectors(
    config: &Collectors,
    registry: &Registry,
) -> anyhow::Result<Vec<Box<dyn Collector>>> {
    let mut collectors = vec![];

    // TODO

    Ok(collectors)
}
