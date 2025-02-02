use serde::{Deserialize, Serialize};
use sysinfo::{Networks, System};

#[derive(Serialize, Deserialize)]
pub struct NetworkInterfaceGroup {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mac_address: String,
    pub packets_received: u64,
    pub packets_transmitted: u64,
}

pub fn get_network_information(system: &mut System) -> NetworkInterfaceGroup {
    system.refresh_all();
    let networks = Networks::new_with_refreshed_list();

    let mut interfaces: Vec<NetworkInterface> = vec![];

    for (interface_name, data) in &networks {
        interfaces.push(NetworkInterface {
            name: interface_name.to_string(),
            received: data.received(),
            transmitted: data.transmitted(),
            mac_address: data.mac_address().to_string(),
            packets_received: data.packets_received(),
            packets_transmitted: data.packets_transmitted(),
        });
    }
    NetworkInterfaceGroup { interfaces }
}
