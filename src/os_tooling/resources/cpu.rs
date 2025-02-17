use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, System};

#[derive(Serialize,Clone, Deserialize,Default,Debug,PartialEq)]
pub struct CPUGroup {
    pub cpus: Vec<CPUResource>,
}
#[derive(Serialize,Clone, Default,Deserialize,Debug,PartialEq)]
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

    system.refresh_specifics(
        sysinfo::RefreshKind::everything().with_cpu(CpuRefreshKind::everything()),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    system.refresh_cpu_all();

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
