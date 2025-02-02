use sysinfo::{Networks, System};

pub fn get_network_information() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let networks = Networks::new_with_refreshed_list();
    let users = sysinfo::Users::new();
    for user in users.list() {
        println!("{} is in {} groups", user.name(), user.groups().len());
    }
    println!("=> networks:");
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {}/{} B  Mac Address: {}, Packets {}/{}",
            data.received(),
            data.transmitted(),
            data.mac_address(),
            data.packets_received(),
            data.packets_transmitted()
        );
    }
}