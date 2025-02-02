use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, System};

#[derive(Serialize, Deserialize)]
pub struct CPUGroup {
    pub cpus: Vec<CPUResource>,
}
#[derive(Serialize, Deserialize)]
pub struct CPUResource {
    pub name: String,
    pub brand: String,
    pub frequency: u64,
    pub usage: f32,
}

impl CPUResource {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn get_current_cpu_usage(system: &mut System) -> CPUGroup {
    println!("Get Current CPU Start");

    system.refresh_specifics(
        sysinfo::RefreshKind::everything().with_cpu(CpuRefreshKind::everything()),
    );

    // Wait a bit because CPU usage is based on diff.
    println!("Get Current CPU Sleep");
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    println!("Get Current CPU Refresh");
    system.refresh_cpu_all();
    println!("Get Current CPU Logic");

    let mut resp: Vec<CPUResource> = vec![];
    for cpu in system.cpus() {
        resp.push(CPUResource {
            name: cpu.name().to_string(),
            brand: cpu.brand().to_string(),
            frequency: cpu.frequency(),
            usage: cpu.cpu_usage(),
        })
    }
    CPUGroup { cpus: resp }
}
