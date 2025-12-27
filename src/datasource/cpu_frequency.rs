use crate::datasource::Reader;
use crate::metrics::cpu_frequency::{CpuFreqStats, DataSource};

pub struct CpuFrequency<R> {
    reader: R,
}

impl<R> CpuFrequency<R>
where
    R: Reader,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> DataSource for CpuFrequency<R>
where
    R: Reader,
{
    fn cpy_freq(&self) -> impl Future<Output = anyhow::Result<CpuFreqStats>> + Send {
        async move {
            let mut core_freq = Vec::new();

            for core in 0..256 {
                let path = format!(
                    "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
                    core
                );

                match self.reader.read_to_string(&path).await {
                    Ok(content) => {
                        let freq = content.trim().parse::<u64>().unwrap_or_else(|_| {
                            tracing::error!("Failed to parse teh CPU frequency for core {}", core);
                            0
                        }) * 1000;
                        core_freq.push(freq);
                    }

                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        // There are no more cores to process
                        break;
                    }

                    Err(e) => return Err(anyhow::anyhow!("Failed to read CPU {}: {}", core, e)),
                }
            }

            if core_freq.is_empty() {
                return Err(anyhow::anyhow!("No CPU frequency sensors found"));
            }

            Ok(CpuFreqStats { cores: core_freq })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::datasource::cpu_frequency::CpuFrequency;
    use crate::datasource::tests::HardcodedReader;
    use crate::metrics::cpu_frequency::DataSource;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_cpu_frequency() {
        let mut test_data = HashMap::new();
        test_data.insert(cpu_freq_path(0), format!("{}", 1100980));
        test_data.insert(cpu_freq_path(1), format!("{}", 883485));
        test_data.insert(cpu_freq_path(2), format!("{}", 4203950));
        test_data.insert(cpu_freq_path(3), format!("{}", 5100362));

        let reader = HardcodedReader::new(test_data);
        let ds = CpuFrequency::new(reader);

        let stats = ds.cpy_freq().await.unwrap();
        assert_eq!(4, stats.cores.len());
        assert_eq!(1000 * 1100980, stats.cores[0]);
        assert_eq!(1000 * 883485, stats.cores[1]);
        assert_eq!(1000 * 4203950, stats.cores[2]);
        assert_eq!(1000 * 5100362, stats.cores[3]);
    }

    fn cpu_freq_path(cpu: usize) -> String {
        format!(
            "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq",
            cpu
        )
    }
}
