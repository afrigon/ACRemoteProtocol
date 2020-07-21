#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Temperature {
    Celcius(u8),
    Fahrenheit(u8),
}

impl Temperature {
    pub fn as_fahrenheit(&self) -> Self {
        match self {
            Temperature::Celcius(temp) => Temperature::Fahrenheit(temp * 9 / 5 + 32),
            Temperature::Fahrenheit(_) => self.clone(),
        }
    }

    pub fn as_celcius(&self) -> Self {
        match self {
            Temperature::Celcius(_) => self.clone(),
            Temperature::Fahrenheit(temp) => Temperature::Celcius((temp - 32) * 5 / 9),
        }
    }
}
