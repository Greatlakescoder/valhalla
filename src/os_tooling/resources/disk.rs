use serde::{Deserialize, Serialize};
use sysinfo::{Disks, RefreshKind, System};

#[derive(Serialize, Deserialize)]
pub struct DiskGroup {
    pub disks: Vec<DiskResource>,
}

#[derive(Serialize, Deserialize)]
pub struct DiskResource {
    pub total: f64,
    pub used: f64,
    pub available: f64,
    pub usage: f64,
}

pub fn get_disk_usage() -> DiskGroup {
    System::new_all();

    // We display all disks' information:

    let disks = Disks::new_with_refreshed_list();
    let mut disk_group: Vec<DiskResource> = vec![];
    for disk in &disks {
        println!("Disk: {}", disk.name().to_string_lossy());
        println!("  Type: {:?}", disk.kind());
        println!("  File System: {}", disk.file_system().to_string_lossy());
        println!("  Mount point: {}", disk.mount_point().display());

        // Get sizes in GB for readability
        let total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_gb =
            (disk.total_space() - disk.available_space()) as f64 / 1024.0 / 1024.0 / 1024.0;
        let available_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;

        println!("  Total: {:.2} GB", total_gb);
        println!("  Used: {:.2} GB", used_gb);
        println!("  Available: {:.2} GB", available_gb);
        println!("  Usage: {:.1}%", (used_gb / total_gb) * 100.0);
        println!();
        disk_group.push(DiskResource {
            total: total_gb,
            used: used_gb,
            available: available_gb,
            usage: ((used_gb / total_gb) * 100.0),
        });
    }

    return DiskGroup { disks: disk_group };
}
