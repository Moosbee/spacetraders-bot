use space_traders_client::models::Agent;

use crate::ship::MyShip;

struct WsObject {
    data: WsData,
}

enum WsData {
    RustShip(MyShip),
    MyAgent(Agent),
}
