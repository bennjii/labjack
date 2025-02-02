//! Conversions obtained from the NIST coefficients
//! for thermocouples, seen [here](http://myweb.astate.edu/sharan/PMC/Labs/Measuring%20Temperature%20with%20Thermocouples.pdf).

use crate::core::{adc::Adc, dac::Dac};
use crate::prelude::LabJackDataValue;

pub enum LoadCell {
    Standard,
}

struct LoadCellContext {
    pub max_rated_load: f64,     // in KG
    pub sensitivity: f64,        // mV/V
    pub excitation_voltage: f64, // in V
}

impl Adc<LoadCellContext> for LoadCell {
    type Digital = f64;

    #[inline]
    fn to_digital(&self, context: LoadCellContext, voltage: LabJackDataValue) -> Self::Digital {
        let denominator =
            context.max_rated_load / (context.sensitivity * context.excitation_voltage);
        self.temp_from_volt(&voltage, &denominator)
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
        let temperature = Thermocouple::TypeT.temp_from_volt(&voltage);

        // Converts to 25.2120 degrees C
        assert_close(temperature, 25.2120);
    }

    #[test]
    fn test_temp_to_volt() {
        let temperature = 25.2120;
        let voltage = Thermocouple::TypeT.volt_from_temp(&temperature);

        // Verifies that the conversion is correct
        assert_close(voltage, 1.0e-3)
    }
}
