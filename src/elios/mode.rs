#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EliosMode {
    Cold = 0b000,
    Dry = 0b001,
    Automatic = 0b010,
    Heat = 0b011,
    Fan = 0b100,
}
