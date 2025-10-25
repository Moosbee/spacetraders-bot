pub struct ShipAssignment {
    pub id: i64,
    pub fleet_id: i64,
    pub disabled: bool,
    pub range_min: i32, // aka fuel capacity minimum
    pub range_max: i32, // aka fuel capacity maximum
    pub cargo_min: i32,
    pub cargo_max: i32,
    pub survey: bool,
    pub extractor: bool,
    pub siphon: bool,
}
