use tokio::time::Duration;

pub const READ_THROTTLE: Duration = Duration::from_nanos(0);
pub const WRITE_THROTTLE: Duration = Duration::from_nanos(1);
pub const READ_TIMEOUT: Duration = Duration::from_millis(10);
pub const READERS: usize = 2;
pub const READ_BATCH: i32 = 100_000;
pub const READ_ITERS: i32 = 100_000_000;
pub const WRITE_BATCH: i32 = 10_000;
pub const WRITE_ITERS: i32 = 100_000;
