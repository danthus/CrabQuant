use std::time::SystemTime;

#[cfg(timeit)]
fn log_time(label: &str, start_time: std::time::SystemTime) {
    let elapsed = start_time.elapsed().expect("Time went backwards");
    println!("[Timer] {}: {} ms elapsed", label, elapsed.as_millis());
}