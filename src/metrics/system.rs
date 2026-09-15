use sysinfo::{Disks, Networks, System};

use crate::snapshot::{SharedSnapshot, Snapshot};

pub struct SystemCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
}

impl SystemCollector {
    pub fn add() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            networks,
        }
    }

    pub fn refresh(&mut self, snapshot: &SharedSnapshot) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh();
        self.networks.refresh();

        let cpu = self.sys.global_cpu_usage().clamp(0.0, 100.0);
        let ram_used = self.sys.used_memory();
        let ram_total = self.sys.total_memory().max(1);
        let ram_pct = (ram_used as f64 / ram_total as f64) as f32 * 100.0;

        let mut disk_total = 0u64;
        let mut disk_free = 0u64;
        for disk in self.disks.list() {
            let fs = disk.file_system().to_string_lossy().to_lowercase();
            if fs.contains("tmpfs") || fs.contains("devfs") || fs.contains("squash") {
                continue;
            }
            disk_total = disk_total.saturating_add(disk.total_space());
            disk_free = disk_free.saturating_add(disk.available_space());
        }
        disk_total = disk_total.max(1);
        let disk_used = disk_total.saturating_sub(disk_free);

        let mut down = 0u64;
        let mut up = 0u64;
        for (_name, data) in self.networks.list() {
            down = down.saturating_add(data.received());
            up = up.saturating_add(data.transmitted());
        }

        if let Ok(mut snap) = snapshot.lock() {
            snap.cpu_pct = cpu;
            Snapshot::push(&mut snap.cpu_history, cpu);
            snap.ram_used = ram_used;
            snap.ram_total = ram_total;
            Snapshot::push(&mut snap.ram_history, ram_pct);
            snap.disk_used = disk_used;
            snap.disk_free = disk_free;
            snap.disk_total = disk_total;
            snap.net_down_bps = down as f64;
            snap.net_up_bps = up as f64;
            Snapshot::push(&mut snap.net_down_history, down as f32);
            Snapshot::push(&mut snap.net_up_history, up as f32);
        }
    }
}
