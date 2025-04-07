use std::{num::NonZeroU32, sync::atomic::AtomicI64, time::Duration};

#[derive(Debug)]
pub struct PriorityRateLimiter {
    pub limiter: governor::RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::QuantaClock,
        governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
    >,
    pub counter: AtomicI64,
}

impl PriorityRateLimiter {
    pub fn new(quota: u64, burst: NonZeroU32) -> Self {
        let quota = governor::Quota::with_period(Duration::from_millis(quota))
            .unwrap()
            .allow_burst(burst);
        // let quota = Quota::per_second(NonZeroU32::new(2).unwrap());

        // let store = DashMapStateStore::new();
        let limiter = governor::RateLimiter::direct(quota);
        Self {
            limiter,
            counter: AtomicI64::new(0),
        }
    }

    pub async fn until_ready(&self, priority: u32, message: &str) {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.limiter.until_ready().await;
        self.counter
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_counter(&self) -> i64 {
        self.counter.load(std::sync::atomic::Ordering::SeqCst)
    }
}
