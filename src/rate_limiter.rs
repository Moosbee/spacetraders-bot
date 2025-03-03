use std::{num::NonZeroU32, time::Duration};

#[derive(Debug)]
pub struct PriorityRateLimiter {
    pub limiter: governor::RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::QuantaClock,
        governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
    >,
}

impl PriorityRateLimiter {
    pub fn new(quota: u64, burst: NonZeroU32) -> Self {
        let quota = governor::Quota::with_period(Duration::from_millis(quota))
            .unwrap()
            .allow_burst(burst);
        // let quota = Quota::per_second(NonZeroU32::new(2).unwrap());

        // let store = DashMapStateStore::new();
        let limiter = governor::RateLimiter::direct(quota);
        Self { limiter }
    }

    pub async fn until_ready(&self, priority: u32, message: &str) {
        self.limiter.until_ready().await;
    }
}
