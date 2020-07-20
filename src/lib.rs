#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FanSpeed {
    Off = 0b000,
    Automatic = 0b100,
    Low = 0b001,
    Medium = 0b010,
    High = 0b011,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Cold = 0b000,
    Dry = 0b001,
    Automatic = 0b010,
    Heat = 0b011,
    Fan = 0b100,
}

const MIN_CELCIUS: u8 = 17;
const MAX_CELCIUS: u8 = 30;
const MIN_FAHRENHEIT: u8 = 62;
const MAX_FAHRENHEIT: u8 = 86;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Temperature {
    Celcius(u8),
    Fahrenheit(u8),
}

fn clamp(temperature: Temperature) -> Temperature {
    match temperature {
        Temperature::Celcius(temp) => Temperature::Celcius(temp.min(MAX_CELCIUS).max(MIN_CELCIUS)),
        Temperature::Fahrenheit(temp) => {
            Temperature::Fahrenheit(temp.min(MAX_FAHRENHEIT).max(MIN_FAHRENHEIT))
        }
    }
}

fn checksum(data: [u8; 5]) -> u8 {
    let data: Vec<u8> = data.iter().map(bitreverse).collect();

    let left = (data[0] ^ data[1] ^ data[2] ^ 0b100) & 0xf;
    let right = ((data[0] >> 4) + (data[1] >> 4) + (data[2] >> 4)) & 0xf;

    let value = !right << 4 | left;

    bitreverse(&value)
}

fn bitreverse(x: &u8) -> u8 {
    (0..8_u8).fold(0, |acc, i| acc | (x >> i & 1) << (7 - i))
}

const FAN_TEMPERATURE: u8 = 0b11110;

#[derive(Debug, Copy, Clone)]
pub struct State {
    fan_speed: FanSpeed,
    mode: Mode,
    temperature: Temperature,
    powered: bool,
    sleep: bool,
}

impl State {
    pub fn new(
        fan_speed: Option<FanSpeed>,
        mode: Mode,
        temperature: Option<Temperature>,
        powered: bool,
        sleep: bool,
    ) -> Option<Self> {
        let temperature = if mode == Mode::Fan {
            if temperature.is_some() {
                return None;
            }

            Temperature::Celcius(MIN_CELCIUS + FAN_TEMPERATURE)
        } else {
            if temperature.is_none() {
                return None;
            }

            clamp(temperature.unwrap())
        };

        let fan = match mode {
            Mode::Automatic | Mode::Dry => {
                if fan_speed.is_some() && fan_speed.unwrap() != FanSpeed::Off {
                    return None;
                }

                FanSpeed::Off
            }
            _ => fan_speed.unwrap_or(FanSpeed::Automatic),
        };

        let sleep = sleep && (mode == Mode::Cold || mode == Mode::Heat || mode == Mode::Automatic);

        Some(Self {
            fan_speed: fan,
            mode,
            temperature,
            powered,
            sleep,
        })
    }

    fn as_raw_parts(self) -> [u8; 5] {
        let mut data: [u8; 5] = [0; 5];

        // header
        data[0] = 0b10100001;

        // options
        data[1] = (self.powered as u8) << 7
            | (self.sleep as u8) << 6
            | (self.fan_speed as u8) << 3
            | (self.mode as u8);

        // temperature
        data[2] = 1 << 6 // unknown 2 bit value
            | (match self.temperature {
                Temperature::Celcius(temp) => temp - MIN_CELCIUS,
                Temperature::Fahrenheit(temp) => temp - MIN_FAHRENHEIT | 0b1 << 5,
            } as u8);

        // timer off
        data[3] = 0b11111111;

        // timer on
        data[4] = 0b11111111;

        data
    }

    pub fn as_value(self) -> u64 {
        let data = self.as_raw_parts();
        let checksum = checksum(data);

        data.iter().fold(0, |acc, x| acc << 8 | *x as u64) << 8 | checksum as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_cold_auto_17c_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Celcius(17)),
                true,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_10100000_01000000_11111111_11111111_01101110,
        );
    }

    #[test]
    fn given_cold_auto_18c_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Celcius(18)),
                true,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_10100000_01000001_11111111_11111111_01101111,
        );
    }

    #[test]
    fn given_cold_auto_62f_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Fahrenheit(62)),
                true,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_10100000_01100000_11111111_11111111_01001110,
        );
    }

    #[test]
    fn given_cold_auto_17c_off_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Celcius(17)),
                false,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_00100000_01000000_11111111_11111111_11101110
        );
    }

    #[test]
    fn given_cold_auto_17c_on_sleeping_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Celcius(17)),
                true,
                true,
            )
            .unwrap()
            .as_value(),
            0b10100001_11100000_01000000_11111111_11111111_00101110
        );
    }

    #[test]
    fn given_heat_auto_30c_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Heat,
                Some(Temperature::Celcius(30)),
                true,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_10100011_01001101_11111111_11111111_01100000
        );
    }

    #[test]
    fn given_fan_auto_on_state_then_value_is_properly_computed() {
        println!(
            "{:b}",
            State::new(Some(FanSpeed::Automatic), Mode::Fan, None, true, false)
                .unwrap()
                .as_value()
        );

        assert_eq!(
            State::new(Some(FanSpeed::Automatic), Mode::Fan, None, true, false)
                .unwrap()
                .as_value(),
            0b10100001_10100100_01011110_11111111_11111111_01111011
        );
    }

    #[test]
    fn given_dry_30c_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(None, Mode::Dry, Some(Temperature::Celcius(30)), true, false,)
                .unwrap()
                .as_value(),
            0b10100001_10000001_01001101_11111111_11111111_01010010
        );
    }

    #[test]
    fn given_cold_auto_78f_on_state_then_value_is_properly_computed() {
        assert_eq!(
            State::new(
                Some(FanSpeed::Automatic),
                Mode::Cold,
                Some(Temperature::Fahrenheit(78)),
                true,
                false,
            )
            .unwrap()
            .as_value(),
            0b10100001_10100000_01110000_11111111_11111111_01010110
        );
    }

    #[test]
    fn when_auto_mode_then_cannot_select_fan_speed() {
        let state = State::new(
            Some(FanSpeed::High),
            Mode::Automatic,
            Some(Temperature::Celcius(24)),
            true,
            false,
        );

        assert!(state.is_none())
    }

    #[test]
    fn when_fan_mode_then_cannot_select_temperature() {
        let state = State::new(
            Some(FanSpeed::Low),
            Mode::Fan,
            Some(Temperature::Celcius(24)),
            true,
            true,
        );

        assert!(state.is_none())
    }

    #[test]
    fn when_dry_mode_then_sleep_is_unavailable() {
        let state =
            State::new(None, Mode::Dry, Some(Temperature::Celcius(24)), true, true).unwrap();

        assert_eq!(state.sleep, false);
    }

    #[test]
    fn when_fan_mode_then_sleep_is_unavailable() {
        let state = State::new(Some(FanSpeed::Low), Mode::Fan, None, true, true).unwrap();

        assert_eq!(state.sleep, false);
    }

    #[test]
    fn given_out_of_range_temperature_then_temperature_is_clamped() {
        let lower_min_celcius = State::new(
            Some(FanSpeed::Automatic),
            Mode::Cold,
            Some(Temperature::Celcius(MIN_CELCIUS - 1)),
            true,
            false,
        )
        .unwrap();
        let higher_max_celcius = State::new(
            Some(FanSpeed::Automatic),
            Mode::Cold,
            Some(Temperature::Celcius(MAX_CELCIUS + 1)),
            true,
            false,
        )
        .unwrap();

        let lower_min_fahrenheit = State::new(
            Some(FanSpeed::Automatic),
            Mode::Cold,
            Some(Temperature::Fahrenheit(MIN_FAHRENHEIT - 1)),
            true,
            false,
        )
        .unwrap();
        let higher_max_fahrenheit = State::new(
            Some(FanSpeed::Automatic),
            Mode::Cold,
            Some(Temperature::Fahrenheit(MAX_FAHRENHEIT + 1)),
            true,
            false,
        )
        .unwrap();

        assert_eq!(
            lower_min_celcius.temperature,
            Temperature::Celcius(MIN_CELCIUS)
        );
        assert_eq!(
            higher_max_celcius.temperature,
            Temperature::Celcius(MAX_CELCIUS)
        );

        assert_eq!(
            lower_min_fahrenheit.temperature,
            Temperature::Fahrenheit(MIN_FAHRENHEIT)
        );
        assert_eq!(
            higher_max_fahrenheit.temperature,
            Temperature::Fahrenheit(MAX_FAHRENHEIT)
        );
    }
}
