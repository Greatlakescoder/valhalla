use std::fs;
use anyhow::Result;


pub fn get_process_fd_count(pid: u32) -> Result<u32> {
    // Path to the process's fd directory
    let fd_path = format!("/proc/{}/fd", pid);
    
    // Read all file descriptors
    let fd_dir = fs::read_dir(&fd_path)?;
    let mut open_files = Vec::new();
    
    // Collect information about each file descriptor
    for entry in fd_dir {
        if let Ok(entry) = entry {
            if let Ok(target) = fs::read_link(entry.path()) {
                open_files.push(target.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(open_files.len() as u32)
   
}

// pub fn analyze_fd_patterns(history: &[ProcessInfo]) -> HashMap<String, usize> {
//     let mut patterns = HashMap::new();
    
//     // Look for suspicious patterns
//     for info in history {
//         // Check for high number of file descriptors
//         if info.fd_count > 1000 {
//             *patterns.entry("high_fd_count".to_string()).or_insert(0) += 1;
//         }
        
//         // Check for sensitive file access
//         for file in &info.open_files {
//             if file.contains("/etc/") || file.contains("/root/") {
//                 *patterns.entry("sensitive_files".to_string()).or_insert(0) += 1;
//             }
//         }
        
//         // Check for many network connections
//         let network_count = info.open_files
//             .iter()
//             .filter(|f| f.contains("socket:"))
//             .count();
            
//         if network_count > 100 {
//             *patterns.entry("high_network_connections".to_string()).or_insert(0) += 1;
//         }
//     }
    
//     patterns
// }