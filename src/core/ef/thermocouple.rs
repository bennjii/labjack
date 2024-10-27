//! Conversions obtained from the NIST coefficients
//! for thermocouples, seen [here](http://myweb.astate.edu/sharan/PMC/Labs/Measuring%20Temperature%20with%20Thermocouples.pdf).

pub struct Thermocouple(ThermocoupleType);

pub enum ThermocoupleType {
    TypeE,
    TypeJ,
    TypeK,
    TypeR,
    TypeS,
    TypeT,
}

impl ThermocoupleType {
    pub(crate) const fn voltage_coefficients(&self) -> &'static [f64] {
        match self {
            // Error +/- 0.02 degrees C
            ThermocoupleType::TypeE => &[
                0.0,
                1.7056035e-2,
                -2.330179e-7,
                6.5435585e-13,
                -7.3562749e-17,
                -1.7896001e-21,
                8.4036165e-26,
                -1.3735879e-30,
                1.0629283e-35,
                -3.2447087e-41,
            ],
            // Error +/- 0.05 degrees C
            ThermocoupleType::TypeJ => &[
                0.0,
                1.978425e-2,
                -2.001204e-7,
                1.036969e-11,
                -2.549687e-16,
                3.585153e-21,
                -5.344285e-26,
                5.099890e-31,
            ],
            // Error +/- 0.05 degrees C
            ThermocoupleType::TypeK => &[
                0.0,
                2.508355e-2,
                7.860106e-8,
                -2.503131e-10,
                8.315270e-14,
                -1.228034e-17,
                9.804036e-22,
                -4.413030e-26,
                1.057734e-30,
                -1.052755e-35,
            ],
            // Error +/- 0.02 degrees C
            ThermocoupleType::TypeR => &[
                0.0,
                1.8891380e-1,
                -9.3835290e-5,
                1.3068619e-7,
                -2.2703580e-10,
                3.5145659e-13,
                -3.8953900e-16,
                2.8239471e-19,
                -1.2607281e-22,
                3.1353611e-26,
                -3.3187769e-30,
            ],
            // Error +/- 0.02 degrees C
            ThermocoupleType::TypeS => &[
                0.0,
                1.84949460e-1,
                -8.00504062e-5,
                1.02237430e-7,
                -1.52248592e-10,
                1.88821343e-13,
                -1.59085941e-16,
                8.23027880e-20,
                -2.34181944e-23,
                2.79786260e-27,
            ],
            // Error +/- 0.03 degrees C
            ThermocoupleType::TypeT => &[
                0.0,
                2.592800e-2,
                -7.602961e-7,
                4.637791e-11,
                -2.165394e-15,
                6.048144e-20,
                -7.293422e-25
            ],
        }
    }

    pub(crate) const fn temperature_coefficients(&self) -> &[f64] {
        match self {
            ThermocoupleType::TypeE => &[
                0.0,
                58.665508710,
                4.503227558e-2,
                2.890840721e-5,
                -3.30568967e-7,
                6.50244033e-10,
                -1.9197496e-13,
                -1.2536600e-15,
                2.14892176e-18,
                -1.4388042e-21,
                3.59608995e-25
            ],
            ThermocoupleType::TypeJ => &[
                0.0,
                50.38118782,
                3.047583693e-2,
                -8.56810657e-5,
                1.322819530e-7,
                -1.7052958e-10,
                2.09480907e-13,
                -1.2538395e-16,
                1.56317257e-20
            ],
            ThermocoupleType::TypeK => &[
                -17.600413686,
                38.921204975,
                1.85587700e-2,
                -9.9457593e-5,
                3.18409457e-7,
                -5.607284e-10,
                5.6075059e-13,
                -3.202072e-16,
                9.7151147e-20,
                -1.210472e-23
            ],
            ThermocoupleType::TypeR => &[
                0.0,
                5.28961729765,
                1.3916658978e-2,
                -2.388556930e-5,
                3.5691600106e-8,
                -4.62347666e-11,
                5.007774410e-14,
                -3.73105886e-17,
                1.577164824e-20,
                -2.81038625e-24
            ],
            ThermocoupleType::TypeS => &[
                0.0,
                5.40313308631,
                1.2593428974e-2,
                -2.324779687e-5,
                3.2202882304e-8,
                -3.314651964e-11,
                2.557442518e-14,
                -1.25068871e-17,
                2.714431761e-21
            ],
            ThermocoupleType::TypeT => &[
                0.0,
                38.748106364,
                3.32922279e-2,
                2.06182434e-4,
                -2.18822568e-6,
                1.09968809e-8,
                -3.0815759e-11,
                4.54791353e-14,
                -2.7512902e-17
            ],
        }
    }
}

impl Thermocouple {
    pub fn temp_from_volt(&self, volt: f64) -> f64 {
        let as_microvolt = volt / 1e-6;
        self.0.voltage_coefficients()
            .iter()
            .enumerate()
            .fold(0.0, |accumulator, (index, coeff)| {
                accumulator + coeff * as_microvolt.powi(index as i32)
            })
    }

    pub fn volt_from_temp(&self, temp: f64) -> f64 {
        let microvolt = self.0.temperature_coefficients()
            .iter()
            .enumerate()
            .fold(0.0, |accumulator, (index, coeff)| {
                accumulator + coeff * temp.powi(index as i32)
            });
        
        microvolt * 1e-6
    }
}

#[cfg(test)]
mod test {
    use crate::core::ef::thermocouple::*;
    
    const CLOSE: f64 = 0.01;
    fn assert_close(value: f64, expected: f64) {
        assert!(value > expected - CLOSE && value < expected + CLOSE)
    }

    #[test]
    fn test_volt_to_temp() {
        // 1mV in Volts
        let voltage = 1.0e-3;
        let temperature = Thermocouple(ThermocoupleType::TypeT).temp_from_volt(voltage);
        
        // Converts to 25.2120 degrees C
        assert_close(temperature, 25.2120);
    }
    
    #[test]
    fn test_temp_to_volt() {
        let temperature = 25.2120;
        let voltage = Thermocouple(ThermocoupleType::TypeT).volt_from_temp(temperature);
        
        // Verifies that the conversion is correct
        assert_close(voltage, 1.0e-3)
    }
}