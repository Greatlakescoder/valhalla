use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

#[derive(Clone,Serialize, Default,Deserialize,Debug,PartialEq)]
pub struct DiskGroup {
    pub disks: Vec<DiskResource>,
}

#[derive(Clone,Serialize, Default,Deserialize,Debug,PartialEq)]
pub struct DiskResource {
    pub total: f64,
    pub used: f64,
    pub available: f64,
    pub usage: f64,
}

pub fn get_disk_usage(system: &mut System) -> DiskGroup {
   

    // We display all disks' information:

    let disks = Disks::new_with_refreshed_list();
    let mut disk_group: Vec<DiskResource> = vec![];
    for disk in &disks {
        // Get sizes in GB for readability
        let total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_gb =
            (disk.total_space() - disk.available_space()) as f64 / 1024.0 / 1024.0 / 1024.0;
        let available_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        disk_group.push(DiskResource {
            total: total_gb,
            used: used_gb,
            available: available_gb,
            usage: ((used_gb / total_gb) * 100.0),
        });
    }

    DiskGroup { disks: disk_group }
}
