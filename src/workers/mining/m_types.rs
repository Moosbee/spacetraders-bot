#[derive(Debug, Default, Clone, Copy, serde::Serialize, PartialEq, Eq)]
pub enum MiningShipAssignment {
    Transporter,
    Extractor,
    Siphoner,
    Surveyor,
    #[default]
    Idle,
    Useless,
}

pub trait SendFuture: core::future::Future {
    fn send(self) -> impl core::future::Future<Output = Self::Output> + Send
    where
        Self: Sized + Send,
    {
        self
    }
}
impl<T: core::future::Future> SendFuture for T {}
