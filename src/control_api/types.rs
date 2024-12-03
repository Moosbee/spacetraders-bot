use crate::{ship::MyShip, sql::Agent};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WsObject {
    pub data: WsData,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsData {
    RustShip(MyShip),
    MyAgent(Agent),
}

pub struct MyReceiver<T: Clone>(pub tokio::sync::broadcast::Receiver<T>);

impl<T: Clone> Clone for MyReceiver<T> {
    fn clone(&self) -> Self {
        MyReceiver(self.0.resubscribe())
    }
}
