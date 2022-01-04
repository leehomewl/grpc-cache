use tokio::time::Duration;

#[derive(Default, Debug)]
pub struct Metrics {
    batch_count: u32,
    batch_duration: Duration,
    batch_avg: Duration,
    batch_max: Duration,
    all_count: usize,
    all_duration: Duration,
    all_avg: Duration,
    timeouts: usize,
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
    pub fn put(&mut self, requests: u32, duration: Duration, timeout: Duration) {
        self.batch_count = requests;
        self.batch_duration = duration;
        self.batch_avg = self.batch_duration / self.batch_count;
        if self.batch_avg > self.batch_max {
            self.batch_max = self.batch_avg;
        }
        self.all_count += requests as usize;
        self.all_duration += duration;
        self.all_avg = self.all_duration / self.all_count as u32;
        if self.batch_avg > timeout {
            self.timeouts += self.batch_count as usize;
        }
    }
}
