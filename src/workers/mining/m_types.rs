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
