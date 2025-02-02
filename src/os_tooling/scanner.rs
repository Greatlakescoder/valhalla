use core::fmt;
use std::{collections::HashMap, ffi::OsString, time::Duration};

use metrics::counter;
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use sysinfo::{CpuRefreshKind, Networks, Pid, ProcessRefreshKind, System};
use thiserror::Error;

use crate::os_tooling::{is_process_alive, OsProcessInformation};

use super::{OsProcessGroup, MetadataTags, ProcessAttribute, ResourceUsageAttribute};
///
/// I think in order to get the model to act how we want, we need to label things and send it to multiple agents
/// We can implement this with a tagging methodolgy similar to how EC2 does it or how you would label data in a csv
/// Tags Available to use
/// - High CPU
/// - High Memory
/// - High Runtime
/// - Has Forked/Spawned Processes
/// We want to store this data in a lookup table so that we can do the post processing of the model out here
/// instead of wasting context windows
///

pub struct SystemScanner {
    attributes: Vec<Box<dyn ProcessAttribute>>,
}

impl SystemScanner {
    pub fn new() -> Self {
        let attributes: Vec<Box<dyn ProcessAttribute>> =
            vec![Box::new(ResourceUsageAttribute::new(60.0, 7200))];
        Self { attributes }
    }

    fn calculate_total_resource_usage(&self, input: &mut OsProcessGroup) {
        let mut total_cpu = input.parent_process.cpu;
        let mut total_memory = input.parent_process.memory_usage;
        let mut total_fd = input.parent_process.fd_count;

        // Sum up resources from all child processes
        for child in &input.forked_threads {
            total_cpu += child.cpu;
            total_memory += child.memory_usage;
            total_fd += child.fd_count;
        }

        // Store totals in parent's attributes
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalCpu, total_cpu.to_string());
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalMemory, total_memory.to_string());
        input
            .parent_process
            .attributes
            .insert(MetadataTags::TotalFileDescriptors, total_fd.to_string());
    }

    pub fn apply_attributes(&self, input: &mut Vec<OsProcessGroup>) {
        for p in input.iter_mut() {
            self.calculate_total_resource_usage(p);
            for attribute in &self.attributes {
                for sp in p.forked_threads.iter_mut() {
                    attribute.tag(sp);
                }
                attribute.tag(&mut p.parent_process);
            }
        }
    }
    /**
    Scans all running processes on the system and groups them by parent-child relationships.
    Returns a Vec of OsProcessGroup where each entry contains a parent process and its forked threads.
    This helps track process hierarchies and identify related processes.
    */
    pub fn scan_running_proccess(&self) -> anyhow::Result<Vec<OsProcessGroup>> {
        tracing::info!("System Monitor scanning");

        let mut agent_output: HashMap<u32, OsProcessGroup> = HashMap::new();
        counter!("scan.run").increment(1);

        let mut sys = System::new_all();
        // Wait a bit because CPU usage is based on diff.
        std::thread::sleep(Duration::from_secs(2));
        // Refresh CPU usage to get actual value.
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );

        for process in sys.processes().values() {
            let formatted_process: OsProcessInformation = process.try_into()?;
            if !is_process_alive(&formatted_process) {
                continue;
            }

            // // Add file descriptors
            // let fd_count = get_process_fd_count(formatted_process.pid)?;
            // formatted_process.fd_count = fd_count as u64;

            // Handle process based on whether it has a parent
            if let Some(parent_pid) = process.parent() {
                let lookup_key = parent_pid.as_u32();

                match sys.process(Pid::from(lookup_key as usize)) {
                    Some(parent_process) if !agent_output.contains_key(&lookup_key) => {
                        // First time seeing this parent, create new entry
                        let formatted_parent: OsProcessInformation = parent_process.try_into()?;
                        agent_output.insert(
                            lookup_key,
                            OsProcessGroup {
                                forked_threads: vec![formatted_process],
                                parent_process: formatted_parent,
                            },
                        );
                    }
                    Some(_) => {
                        // Parent exists in map, update forked threads
                        agent_output
                            .entry(lookup_key)
                            .and_modify(|x| x.forked_threads.push(formatted_process.clone()))
                            .or_insert(OsProcessGroup {
                                forked_threads: vec![],
                                parent_process: formatted_process,
                            });
                    }
                    None => continue,
                }
            } else {
                // No parent, add as standalone process
                agent_output
                    .entry(formatted_process.pid)
                    .or_insert(OsProcessGroup {
                        forked_threads: vec![],
                        parent_process: formatted_process,
                    });
            }
        }
        tracing::info!("System Monitor scanning done");
        // Convert to Vec more efficiently
        Ok(agent_output.into_values().collect())
    }
}

impl Default for SystemScanner {
    fn default() -> Self {
        Self::new()
    }
}
