use std::sync::Arc;

// tests/scanner_test.rs
use crate::helpers::TestEnvironment;
use odin::{configuration::get_configuration, cache::Cache, monitor::SystemMonitor, os_tooling::process::OsProcessGroup};
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use odin::configuration::Settings;
    use tokio::test;

    #[test]
    async fn test_system_monitor_snapshot() {
        let settings = Settings::default(); // You'll need to implement this
        let monitor = SystemMonitor::new(settings);

        // Get initial snapshot
        let snapshot = monitor.get_latest_snapshot().await;
        
        // Basic sanity checks
        assert!(!snapshot.processes.is_empty(), "Should have some processes");
        assert!(snapshot.memory.total_memory > 0, "Should have non-zero total memory");
        assert!(!snapshot.disks.disks.is_empty(), "Should have some disk info");
    }

    #[test]
    async fn test_monitor_continuous_updates() {
        let settings = Settings::default();
        let monitor = SystemMonitor::new(settings);
        
        // Spawn the monitor
        let sub_monitor = monitor.clone();
        let monitor_handle = tokio::spawn(async move {
            sub_monitor.run().await.expect("Monitor should run");
        });

        // Wait a bit for data collection
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Get snapshots and verify they're updating
        let snapshot1 = monitor.get_latest_snapshot().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let snapshot2 = monitor.get_latest_snapshot().await;
        
        assert_ne!(
            snapshot1.cpu, 
            snapshot2.cpu, 
            "CPU metrics should update"
        );
        
        // Cleanup
        monitor_handle.abort();
    }
}