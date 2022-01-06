use tokio::time::Duration;

pub const READ_THROTTLE: Duration = Duration::from_nanos(0);
pub const WRITE_THROTTLE: Duration = Duration::from_nanos(0);
pub const READ_TIMEOUT: Duration = Duration::from_millis(3);
pub const READERS: usize = 3;
pub const READ_REPORT: usize = 2_000_000 / BATCH_SIZE;
pub const READ_ITERS: usize = 10_000_000;
pub const WRITE_ITERS: i32 = 5_000_000;
pub const WRITE_FLUSH: i32 = 5_000_000;
pub const BATCH_SIZE: usize = 10;
