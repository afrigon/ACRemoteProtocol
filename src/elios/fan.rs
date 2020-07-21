#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EliosFanSpeed {
    Off = 0b000,
    Automatic = 0b100,
    Low = 0b001,
    Medium = 0b010,
    High = 0b011,
}
