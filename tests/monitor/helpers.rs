// tests/helper.rs
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
/// Different types of test processes we can spawn
#[derive(Debug)]
pub enum TestProcessType {
    Idle,   // Just sits there doing nothing
    Sleep,  // Sleeps continuously
    Active, // Keeps CPU busy
}

/// Represents a single test process
pub struct TestProcess {
    child: Child,
    process_type: TestProcessType,
}

impl TestProcess {
    pub fn spawn(prefix: String, process_type: TestProcessType) -> Self {
        let child = match process_type {
            TestProcessType::Idle => Command::new("sh")
                .arg("-c")
                .arg(format!("echo '{}' && echo", prefix + "idle"))
                .spawn(),
            TestProcessType::Sleep => Command::new("sleep")
                .arg("3600")
                .spawn(),
            TestProcessType::Active => Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "echo '{}' && while true; do echo 1 > /dev/null; done",
                    prefix + "Active"
                ))
                .spawn(),
        }
        .expect("Failed to spawn test process");

        Self {
            child,
            process_type,
        }
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

}

impl Drop for TestProcess {
    fn drop(&mut self) {
        self.child.kill().expect("Process could not be killed");
        println!("{} has been killed",self.pid())
    }
}

/// Test environment that manages multiple test processes
pub struct TestEnvironment {
    processes: Vec<TestProcess>,
    process_prefix: String,
}

impl TestEnvironment {
    pub fn setup(process_prefix: String) -> Self {
        // Create one of each type of process
        let processes = vec![
            TestProcess::spawn(process_prefix.clone(), TestProcessType::Idle),
            TestProcess::spawn(process_prefix.clone(), TestProcessType::Sleep),
            TestProcess::spawn(process_prefix.clone(), TestProcessType::Active),
        ];

        // Give processes time to start up
        thread::sleep(Duration::from_millis(500));

        Self {
            process_prefix,
            processes,
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Cleanup happens automatically through TestProcess Drop impl
    }
}
