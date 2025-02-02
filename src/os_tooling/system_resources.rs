use core::fmt;
use std::{collections::HashMap, ffi::OsString};

use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use sysinfo::{CpuRefreshKind, Networks, System};
use thiserror::Error;


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemInformation {
    pub name: String,
    pub os_version: String,
    pub host_name: String,
    pub uptime: u64,
    pub total_cpus: u64,
    pub total_memory: u64,
    pub cpu_arch: String,
}

impl fmt::Display for SystemInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Name {} \n 
            Os Version {} \n
            Hostname {} \n
            uptime {} \n
            total cpus {} \n
            total memory {} \n
            cpu arch {} \n",
            self.name,
            self.os_version,
            self.host_name,
            self.uptime,
            self.total_cpus,
            self.total_memory,
            self.cpu_arch
        )
    }
}



