use sysinfo::{System, SystemExt};
use std::{thread, time::Duration};

fn main() {
    let mut sys = System::new_all();
    // We need to wait a bit to get an accurate CPU usage
    thread::sleep(Duration::from_millis(500));
    sys.refresh_cpu();
    println!("CPU Usage: {:.2}%", sys.global_cpu_info().cpu_usage());
}
