// #[cfg(timestamp_test)]
// fn log_time(label: &str, start_time: std::time::SystemTime) {
//     let elapsed = start_time.elapsed().expect("Time went backwards");
//     println!("[Timer] {}: {} ms elapsed", label, elapsed.as_millis());
// }

pub struct Counter {
    count: u64,
}

impl Counter {
    pub fn new() -> Self {
        Counter { count: 0 }
    }

    pub fn next(&mut self) -> u64 {
        self.count += 1;
        self.count
    }
}