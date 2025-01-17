// tests/scanner_test.rs
use crate::helpers::TestEnvironment;
use odin::{configuration::get_configuration, monitor::SystemMonitor};

#[test]
fn test_process_scanning() {
    // Set a specific prefix for this test
    let configuration = {
        
        get_configuration().expect("Failed to read configuration.")
    };
    let system_scanner = SystemMonitor::new(configuration);
    TestEnvironment::setup("Loki".to_string());
    let processes_found = system_scanner
        .collect_info()
        .expect("Failed to collect test proccesses");
    let test_runner_pid = std::process::id();
    for p in processes_found {
        if p.agent_input.parent_process.pid == test_runner_pid {
            // Four forked threads
            // One is this function since cargo isolate tests in own thread
            // Three for the spawned proccesses in Test Env
            assert_eq!(p.agent_input.forked_threads.len(), 4)
        }
    }
}
