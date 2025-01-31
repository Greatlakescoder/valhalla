use std::sync::Arc;

// tests/scanner_test.rs
use crate::helpers::TestEnvironment;
use odin::{configuration::get_configuration, memory::Cache, monitor::SystemMonitor, os_tooling::AgentInput};
use tokio::sync::Mutex;

#[tokio::test]
async fn test_process_scanning() {
    // Set a specific prefix for this test
    let configuration = {
        
        get_configuration().expect("Failed to read configuration.")
    };
    let storage: Cache<String, Vec<AgentInput>> = Cache::new(60);
    let storage = Arc::new(Mutex::new(storage));
    
    let system_scanner = SystemMonitor::new(configuration,storage);
    TestEnvironment::setup("Loki".to_string());
    let processes_found = system_scanner
        .collect_info().await
        .expect("Failed to collect test proccesses");
    let test_runner_pid = std::process::id();
    for p in processes_found {
        if p.parent_process.pid == test_runner_pid {
            // Four forked threads
            // One is this function since cargo isolate tests in own thread
            // Three for the spawned proccesses in Test Env
            assert_eq!(p.forked_threads.len(), 4)
        }
    }
}
