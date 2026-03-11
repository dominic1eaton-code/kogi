use kogi_network_module::NetworkSystem;

fn main() {
    let system = NetworkSystem::with_default_config();
    system.print_topology();
}
