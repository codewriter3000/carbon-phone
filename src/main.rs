use smithay_client_toolkit::{default_environment, new_default_environment};

default_environment!(MyApp, desktop);

fn main() {
    let (environment, display, event_queue) = new_default_environment!(MyApp, desktop)
        .expect("Failed to initialize the Wayland environment.");

    // environment.manager is the underlying GlobalManager
    println!("Available globals:");
    for (name, interface, version) in environment.manager.list() {
        println!("{}: {} (version {})", name, interface, version);
    }
}

