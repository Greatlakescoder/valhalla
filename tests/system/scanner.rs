// tests/scanner_test.rs
use crate::helpers::{TestEnvironment, TestProcess, TestProcessType};
use odin::{configuration::get_configuration, os_tooling::SystemScanner};
use sysinfo::System;

#[test]
fn test_process_scanning() {
    // Set a specific prefix for this test
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.scanner.prefix = Some("Loki".to_owned());
        c
    };
    let system_scanner = SystemScanner::build(&configuration.scanner);
    TestEnvironment::setup(configuration.scanner.prefix.unwrap());
    let processes_found = system_scanner
        .scan_running_proccess()
        .expect("Failed to collect test proccesses");
    let test_runner_pid = std::process::id();
    for p in processes_found {
        if p.parent_process.pid == test_runner_pid {
            println!("PID {}", p.parent_process.pid);
            for ft in &p.forked_threads {
                println!("{} - {} - {:?}", ft.pid, ft.name, ft.command)
            }
            // Four forked threads 
            // One is this function since cargo isolate tests in own thread
            // Three for the spawned proccesses in Test Env
            assert_eq!(p.forked_threads.len(), 4)
        }
    }
}
