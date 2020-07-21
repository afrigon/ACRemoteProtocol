use bit_vec::BitVec;

pub struct InfraredProtocol {
    /// The duration of the beginning pulse in microseconds
    leading_pulse: u32,
    /// The duration of the gap in microseconds after the leading pulse
    leading_gap: u32,
    /// The duration of a pulse in microseconds when sending a logical 1
    one_pulse: u32,
    /// The duration of the gap in microseconds when sending a logical 1
    one_gap: u32,
    /// The duration of a pulse in microseconds when sending a logical 0
    zero_pulse: u32,
    /// The duration of the gap in microseconds when sending a logical 0
    zero_gap: u32,
}

impl InfraredProtocol {
    pub fn encode(&self, data: BitVec) -> Vec<u32> {
        let mut buffer = Vec::new();

        buffer.push(self.leading_pulse);
        buffer.push(self.leading_gap);

        for value in data.iter() {
            if value {
                buffer.push(self.one_pulse);
                buffer.push(self.one_gap);
            } else {
                buffer.push(self.zero_pulse);
                buffer.push(self.zero_gap);
            }
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IR: InfraredProtocol = InfraredProtocol {
        leading_pulse: 4500,
        leading_gap: 4500,
        one_pulse: 500,
        one_gap: 1500,
        zero_pulse: 500,
        zero_gap: 500,
    };

    #[test]
    fn given_simple_data_then_is_encoded_properly() {
        let mut data = BitVec::from_elem(2, false);
        data.set(1, true);

        let result = IR.encode(data);

        assert_eq!(
            result,
            vec!(
                IR.leading_pulse,
                IR.leading_gap,
                IR.zero_pulse,
                IR.zero_gap,
                IR.one_pulse,
                IR.one_gap,
            )
        );
    }
}
