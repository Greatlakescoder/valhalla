use crate::{
    cache::{blob::Cache, get_cached_data},
    configuration::Settings,
    ollama::{OllamaClient, ProcessInfo, ProcessScore},
    os_tooling::{
        cpu::{get_current_cpu_usage, CPUGroup},
        disk::{get_disk_usage, DiskGroup},
        memory::{get_system_memory, SystemMemory},
        network::{get_network_information, NetworkInterfaceGroup},
        process::OsProcessGroup,
        SystemScanner,
    },
};
use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use sysinfo::System;
use tokio::sync::Mutex;

// New analysis store for Ollama results
pub struct AnalysisStore {
    cache: Arc<Mutex<HashMap<u32, ProcessAnalysis>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessAnalysis {
    name: String,
    score: u8,
    reason: String,
}

impl AnalysisStore {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn update(&self, analyses: Vec<ProcessScore>) {
        let mut cache = self.cache.lock().await;
        for analysis in analyses {
            cache.insert(
                analysis.pid,
                ProcessAnalysis {
                    name: analysis.name,
                    score: analysis.score,
                    reason: analysis.reason,
                },
            );
        }
    }

    pub async fn get_analysis(&self, pid: u32) -> Option<ProcessAnalysis> {
        let cache = self.cache.lock().await;
        cache.get(&pid).cloned()
    }
}

// Shared metric storage for each resource type
pub struct MetricStore<T> {
    cache: Arc<Mutex<Cache<String, T>>>,
}

impl<T: Clone + std::fmt::Debug> MetricStore<T> {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(Cache::new(ttl_seconds))),
        }
    }

    pub async fn store(&self, metric_type: &str, value: T) {
        let mut cache = self.cache.lock().await;
        // Include metric type in key to prevent collisions
        let key = format!("{}_{}", metric_type, Local::now());
        cache.insert(key, value);
        // cache.remove_expired();
    }

    pub async fn get_recent(&self, count: usize) -> Vec<T> {
        let cache = self.cache.lock().await;
        get_cached_data(&cache).into_iter().take(count).collect()
    }
}

#[derive(Serialize, Default, Deserialize, Debug)]
pub struct MonitorOutput {
    pub processes: Vec<OsProcessGroup>,
    pub cpu: CPUGroup,
    pub memory: SystemMemory,
    pub disks: DiskGroup,
    pub network: NetworkInterfaceGroup,
}

impl MonitorOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_processes(mut self, processes: Vec<OsProcessGroup>) -> Self {
        self.processes = processes;
        self
    }

    pub fn with_cpu(mut self, cpus: CPUGroup) -> Self {
        self.cpu = cpus;
        self
    }

    pub fn with_memory(mut self, memory: SystemMemory) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_disks(mut self, disks: DiskGroup) -> Self {
        self.disks = disks;
        self
    }

    pub fn with_networks(mut self, networks: NetworkInterfaceGroup) -> Self {
        self.network = networks;
        self
    }
}

// Individual monitor implementations
pub struct ProcessMonitor {
    system: System,
    store: Arc<MetricStore<Vec<OsProcessGroup>>>,
}

impl ProcessMonitor {
    pub fn new(store: Arc<MetricStore<Vec<OsProcessGroup>>>) -> Self {
        Self {
            system: System::new_all(),
            store,
        }
    }

    pub async fn run(mut self, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            match self.collect().await {
                Ok(processes) => {
                    self.store.store("process", processes).await;
                }
                Err(e) => {
                    tracing::error!("Failed to collect process metrics: {}", e);
                }
            }
        }
    }

    async fn collect(&mut self) -> Result<Vec<OsProcessGroup>> {
        let scanner = SystemScanner::new();
        let mut results = scanner.scan_running_proccess()?;
        scanner.apply_attributes(&mut results);
        Ok(results)
    }
}

pub struct CPUMonitor {
    system: System,
    store: Arc<MetricStore<CPUGroup>>,
}

impl CPUMonitor {
    pub fn new(store: Arc<MetricStore<CPUGroup>>) -> Self {
        Self {
            system: System::new_all(),
            store,
        }
    }

    pub async fn run(mut self, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let metrics = self.collect();
            self.store.store("cpu", metrics).await;
        }
    }

    fn collect(&mut self) -> CPUGroup {
        get_current_cpu_usage(&mut self.system)
    }
}

pub struct MemoryMonitor {
    system: System,
    store: Arc<MetricStore<SystemMemory>>,
}

impl MemoryMonitor {
    pub fn new(store: Arc<MetricStore<SystemMemory>>) -> Self {
        Self {
            system: System::new_all(),
            store,
        }
    }

    pub async fn run(mut self, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let metrics = self.collect();
            self.store.store("memory", metrics).await;
        }
    }

    fn collect(&mut self) -> SystemMemory {
        get_system_memory(&mut self.system)
    }
}

pub struct NetworkMonitor {
    system: System,
    store: Arc<MetricStore<NetworkInterfaceGroup>>,
}

impl NetworkMonitor {
    pub fn new(store: Arc<MetricStore<NetworkInterfaceGroup>>) -> Self {
        Self {
            system: System::new_all(),
            store,
        }
    }

    pub async fn run(mut self, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let metrics = self.collect();
            self.store.store("network", metrics).await;
        }
    }

    fn collect(&mut self) -> NetworkInterfaceGroup {
        get_network_information(&mut self.system)
    }
}

pub struct DiskMonitor {
    system: System,
    store: Arc<MetricStore<DiskGroup>>,
}

impl DiskMonitor {
    pub fn new(store: Arc<MetricStore<DiskGroup>>) -> Self {
        Self {
            system: System::new_all(),
            store,
        }
    }

    pub async fn run(mut self, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let metrics = self.collect();
            self.store.store("disk", metrics).await;
        }
    }

    fn collect(&mut self) -> DiskGroup {
        get_disk_usage(&mut self.system)
    }
}

// Main system monitor coordinates everything
#[derive(Clone)]
pub struct SystemMonitor {
    process_store: Arc<MetricStore<Vec<OsProcessGroup>>>,
    cpu_store: Arc<MetricStore<CPUGroup>>,
    memory_store: Arc<MetricStore<SystemMemory>>,
    disk_store: Arc<MetricStore<DiskGroup>>,
    network_store: Arc<MetricStore<NetworkInterfaceGroup>>,
    analysis_store: Arc<AnalysisStore>,
    pub settings: Settings,
}

impl SystemMonitor {
    pub fn new(settings: Settings) -> Self {
        Self {
            process_store: Arc::new(MetricStore::new(30)), // 5 min TTL
            cpu_store: Arc::new(MetricStore::new(10)),     // 1 min TTL
            memory_store: Arc::new(MetricStore::new(10)),
            disk_store: Arc::new(MetricStore::new(300)),
            network_store: Arc::new(MetricStore::new(10)),
            analysis_store: Arc::new(AnalysisStore::new()),
            settings,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Spawn process monitor
        let process_monitor = ProcessMonitor::new(Arc::clone(&self.process_store));
        tokio::spawn(async move {
            // tokio::signal::ctrl_c().await.unwrap();
            process_monitor.run(Duration::from_secs(5)).await;
        });

        // Spawn CPU monitor
        let cpu_monitor = CPUMonitor::new(Arc::clone(&self.cpu_store));
        tokio::spawn(async move {
            // tokio::signal::ctrl_c().await.unwrap();
            cpu_monitor.run(Duration::from_secs(2)).await;
        });

        // Spawn Disk monitor
        let disk_monitor = DiskMonitor::new(Arc::clone(&self.disk_store));
        tokio::spawn(async move {
            // tokio::signal::ctrl_c().await.unwrap();
            disk_monitor.run(Duration::from_secs(10)).await;
        });

        // Spawn Network monitor
        let network_monitor = NetworkMonitor::new(Arc::clone(&self.network_store));
        tokio::spawn(async move {
            // tokio::signal::ctrl_c().await.unwrap();
            network_monitor.run(Duration::from_secs(10)).await;
        });

        // Spawn Memory monitor
        let memory_monitor = MemoryMonitor::new(Arc::clone(&self.memory_store));
        tokio::spawn(async move {
            // tokio::signal::ctrl_c().await.unwrap();
            memory_monitor.run(Duration::from_secs(10)).await;
        });
        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    pub async fn run_analysis(&self, ollama: OllamaClient) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            // Get latest process list
            if let Some(processes) = self.process_store.get_recent(1).await.pop() {
                // Convert to ProcessInfo for Ollama

                // Get analysis from Ollama
                match ollama.analyze_process_names(&processes).await {
                    Ok(scores) => {
                        // Store in analysis store instead of process store
                        tracing::info!("Saving Scores");
                        self.analysis_store.update(scores).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to analyze processes: {}", e);
                    }
                }
            }
        }
    }

    // API endpoint helper
    pub async fn get_latest_snapshot(&self) -> MonitorOutput {
        let processes = self
            .process_store
            .get_recent(1)
            .await
            .pop()
            .unwrap_or_default();

        // Enrich processes with analysis data
        let mut enriched_processes = processes.clone();
        for process in &mut enriched_processes {
            if let Some(analysis) = self
                .analysis_store
                .get_analysis(process.parent_process.pid)
                .await
            {
                tracing::info!("Updating process with threat score {:?}", process.parent_process);
                process.parent_process.attributes.insert(
                    crate::os_tooling::MetadataTags::ThreatScore,
                    analysis.score.to_string(),
                );
                process.parent_process.attributes.insert(
                    crate::os_tooling::MetadataTags::ThreatScoreReason,
                    analysis.reason,
                );
            }
        }

        MonitorOutput::new()
            .with_processes(enriched_processes)
            .with_cpu(self.cpu_store.get_recent(1).await.pop().unwrap_or_default())
            .with_memory(
                self.memory_store
                    .get_recent(1)
                    .await
                    .pop()
                    .unwrap_or_default(),
            )
            .with_disks(
                self.disk_store
                    .get_recent(1)
                    .await
                    .pop()
                    .unwrap_or_default(),
            )
            .with_networks(
                self.network_store
                    .get_recent(1)
                    .await
                    .pop()
                    .unwrap_or_default(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_metric_store_basic_operations() {
        let store = MetricStore::<i32>::new(10); // 10 second TTL

        // Test storing and retrieving
        store.store("test_metric", 42).await;
        let recent = store.get_recent(1).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0], 42);
    }

    #[test]
    async fn test_metric_store_ttl() {
        let store = MetricStore::<String>::new(1); // 1 second TTL

        store.store("test_metric", "data".to_string()).await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let recent = store.get_recent(1).await;
        assert_eq!(recent.len(), 0, "Store should be empty after TTL");
    }

    #[test]
    async fn test_monitor_output_builder() {
        let processes = vec![OsProcessGroup::default()];
        let cpu = CPUGroup::default();
        let memory = SystemMemory::default();
        let disks = DiskGroup::default();
        let network = NetworkInterfaceGroup::default();

        let output = MonitorOutput::new()
            .with_processes(processes.clone())
            .with_cpu(cpu.clone())
            .with_memory(memory.clone())
            .with_disks(disks.clone())
            .with_networks(network.clone());

        assert_eq!(output.processes, processes);
        assert_eq!(output.cpu, cpu);
        assert_eq!(output.memory, memory);
        assert_eq!(output.disks, disks);
        assert_eq!(output.network, network);
    }
}
