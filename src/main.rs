use std::{thread, time::Duration};
use sysinfo::System;

const GB: f64 = 1024.0 * 1024.0 * 1024.0;

fn line(used: u64, total: u64) -> String {
    let pct = used as f64 / total as f64 * 100.0;
    format!("Total: {:.1} GB  Used: {:.1} GB  {:.1}%", total as f64 / GB, used as f64 / GB, pct)
}

fn main() {
    let mut sys = System::new().expect("sysinfo: unsupported platform");
    loop {
        sys.refresh_memory();
        println!("{}", line(sys.used_memory(), sys.total_memory()));
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_gb_and_percent() {
        assert_eq!(line(12_241_000_000, 34_250_000_000), "Total: 31.9 GB  Used: 11.4 GB  35.7%");
    }
}
