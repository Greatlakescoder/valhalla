// tests/scanner_test.rs
use crate::helpers::{TestEnvironment, TestProcess, TestProcessType};
use odin_hackathon::{configuration::get_configuration, os_tooling::SystemScanner};
use sysinfo::System;

#[test]
fn test_process_scanning() {
    // Set a specific prefix for this test
    // std::env::set_var("TEST_PROCESS_PREFIX", "scanner_test");
    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.prefix = Some("Loki".to_owned());
        c
    };
    let system_scanner = SystemScanner::build(&configuration).expect("Could not build scanner");
    let env = TestEnvironment::setup(configuration.prefix.unwrap());
    let processes_found = system_scanner.scan_running_proccess().expect("Failed to collect test proccesses");
    let test_runner_pid = std::process::id();
    assert_eq!(processes_found.len(),2);
    
}