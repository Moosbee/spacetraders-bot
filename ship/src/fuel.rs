#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct FuelState {
    pub capacity: i32,
    pub current: i32,
}

impl FuelState {
    pub fn update(&mut self, data: &space_traders_client::models::ShipFuel) {
        self.current = data.current;
        self.capacity = data.capacity;
    }
}
