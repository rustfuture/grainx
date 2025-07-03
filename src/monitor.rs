use sysinfo::{System, Networks, Pid, Disks};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

pub struct SystemMonitor {
    pub sys: System,
    pub networks: Networks,
    pub disks: Disks,
    pub last_cpu_usage: f32,
    pub high_cpu_duration: u32, // Counter for consecutive high CPU readings
    pub cpu_state_history: VecDeque<bool>, // true for high CPU, false for low
    pub user_cpu_threshold: f32, // User-learned CPU threshold
    pub high_cpu_count: u32, // Counter for how many times CPU exceeds user_cpu_threshold
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            sys: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            last_cpu_usage: 0.0,
            high_cpu_duration: 0,
            cpu_state_history: VecDeque::with_capacity(5),
            user_cpu_threshold: 75.0, // Initial threshold
            high_cpu_count: 0,
        }
    }

    pub fn get_cpu_usage(&mut self) -> f32 {
        self.sys.refresh_cpu();
        thread::sleep(Duration::from_millis(200)); // Initial wait for CPU usage to be calculated
        self.sys.refresh_cpu();
        let current_cpu_usage = self.sys.global_cpu_info().cpu_usage();

        // Adaptive sampling logic
        let sampling_interval_ms = if current_cpu_usage > 80.0 {
            100 // High CPU, sample more frequently
        } else if current_cpu_usage > 50.0 {
            200 // Medium CPU
        } else {
            500 // Low CPU, sample less frequently
        };
        thread::sleep(Duration::from_millis(sampling_interval_ms));

        // Bottleneck identification logic
        if current_cpu_usage > 90.0 {
            self.high_cpu_duration += 1;
            if self.high_cpu_duration >= 5 { // If high CPU for 5 consecutive readings
                println!("\nWARNING: Sustained high CPU usage detected! Potential CPU bottleneck.\n");
            }
        } else {
            self.high_cpu_duration = 0;
        }

        // Behavioral pattern recognition
        let is_high_cpu = current_cpu_usage > 70.0; // Define a threshold for "high"
        self.cpu_state_history.push_back(is_high_cpu);
        if self.cpu_state_history.len() > 5 { // Keep history size limited
            self.cpu_state_history.pop_front();
        }

        // Check for a simple fluctuation pattern: High -> Low -> High
        if self.cpu_state_history.len() == 3 && 
           self.cpu_state_history[0] == true && 
           self.cpu_state_history[1] == false && 
           self.cpu_state_history[2] == true {
            println!("\nPATTERN DETECTED: CPU usage fluctuating (High-Low-High)!\n");
            // Clear history to avoid repeated detection of the same pattern
            self.cpu_state_history.clear();
        }

        // User behavior learning for personalized monitoring
        if current_cpu_usage > self.user_cpu_threshold {
            self.high_cpu_count += 1;
            if self.high_cpu_count >= 10 { // If CPU exceeds user threshold 10 times
                self.user_cpu_threshold += 5.0; // Increase threshold
                println!("\nUSER BEHAVIOR LEARNED: Increased user CPU threshold to {:.1}%\n", self.user_cpu_threshold);
                self.high_cpu_count = 0; // Reset counter
            }
        } else {
            self.high_cpu_count = 0; // Reset if CPU drops below threshold
        }

        self.last_cpu_usage = current_cpu_usage;
        current_cpu_usage
    }

    pub fn get_memory_usage(&mut self) -> (u64, u64) {
        self.sys.refresh_memory();
        (self.sys.used_memory(), self.sys.total_memory())
    }

    pub fn get_processes(&mut self) -> Vec<(usize, String, f32, u64)> {
        self.sys.refresh_processes();
        let mut processes: Vec<(usize, String, f32, u64)> = self.sys.processes()
            .iter()
            .map(|(pid, p)| (pid.as_u32() as usize, p.name().to_string(), p.cpu_usage(), p.memory()))
            .collect();
        processes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        processes.truncate(10); // Top 10 processes (will be limited by config in UI)
        processes
    }

    pub fn kill_process(&mut self, pid: usize) -> bool {
        if let Some(process) = self.sys.process(Pid::from(pid)) {
            process.kill()
        } else {
            false
        }
    }

    pub fn get_network_io(&mut self) -> (u64, u64) {
        self.networks.refresh();
        let mut received_bytes = 0;
        let mut transmitted_bytes = 0;
        for (_interface_name, data) in self.networks.iter() {
            received_bytes += data.received();
            transmitted_bytes += data.transmitted();
        }
        (received_bytes, transmitted_bytes)
    }

    pub fn get_cpu_cores(&mut self) -> Vec<f32> {
        self.sys.refresh_cpu();
        thread::sleep(Duration::from_millis(200));
        self.sys.refresh_cpu();
        self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }

    pub fn get_disk_usage(&mut self) -> Vec<(String, u64, u64, f64)> {
        self.disks.refresh();
        self.disks.iter()
            .map(|disk| {
                let name = disk.name().to_string_lossy().to_string();
                let total = disk.total_space();
                let available = disk.available_space();
                let used_percentage = if total > 0 {
                    ((total - available) as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                (name, total, available, used_percentage)
            })
            .collect()
    }

    pub fn get_system_info(&mut self) -> (String, String, u64) {
        self.sys.refresh_all();
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let uptime = System::uptime();
        (os_name, kernel_version, uptime)
    }
}

