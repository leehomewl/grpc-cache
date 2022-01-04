use tokio::time::Duration;

#[derive(Default, Debug)]
pub struct Metrics {
    batch_count: usize,
    batch_duration: Duration,
    batch_avg: Duration,
    all_count: usize,
    all_duration: Duration,
    all_avg: Duration,
    all_max: Duration,
    timeouts: usize,
    success: f64,
}

// impl Default for Metrics {
//     fn default() -> Self {
//         Self {
//             count: 0,
//             total_duration: Duration::ZERO,
//             max: Duration::ZERO,
//         }
//     }
// }

impl Metrics {
    pub fn put(&mut self, requests: usize, duration: Duration, timeout: Duration) {
        self.batch_count = requests;
        self.batch_duration = duration;
        self.batch_avg = self.batch_duration / self.batch_count as u32;
        if self.batch_duration > self.all_max {
            self.all_max = self.batch_duration;
        }
        self.all_count += requests as usize;
        self.all_duration += duration;
        self.all_avg = self.all_duration / self.all_count as u32;
        if self.batch_duration > timeout {
            self.timeouts += self.batch_count as usize;
        }
        self.success = 100.0 * (1.0 - (self.timeouts as f64 / self.all_count as f64));
    }
}
