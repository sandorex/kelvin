use crate::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, path::{Path, PathBuf}};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SensorMap {
    pub min: f32,
    pub max: f32,
}

impl SensorMap {
    pub fn map(&self, value: f32, min: f32, max: f32) -> f32 {
        return (value - min) * (self.max - self.min) / (max - min) + min;
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Sensor {
    /// Name of the sensor
    pub name: String,

    /// Label shown before the temperature
    pub label: String,

    /// At what temperature should the alarm be triggered, if not set then no alarm will ever
    /// trigger for this sensor
    #[serde(default)]
    pub alarm: Option<f32>,

    /// Maximum value the sensor should go up to
    pub max: f32,

    // /// Midpoint for the sensor value, used for cosmetic purposes
    // midpoint: f32,

    /// Minimum value the sensor should go down to
    pub min: f32,

    /// How many decimals to round the number to (0 meaning an integer)
    #[serde(default)]
    pub round: Option<u8>,

    /// Map the value into a new range (can be used to convert to/from PWM or percentage)
    #[serde(default)]
    pub map: Option<SensorMap>,

    // /// Convert the raw data into something else
    // #[serde(default)]
    // convert: Option<SensorConverter>,

    /// Path of the sensor or sensor sysfs file
    ///
    /// To use lm_sensors use following format:
    ///     @sensors/amdgpu-pci-0300/junction/temp2_input
    pub path: PathBuf,
}

/// Get json value using `Path` with each segment being a key in json object
fn get_by_path<'a>(object: &'a JsonValue, path: &Path) -> Option<&'a JsonValue> {
    let components = path.components().map(|x| x.as_os_str().to_str().unwrap()).collect::<Vec<_>>();

    let mut value: &JsonValue = object;
    for component in components {
        if let Some(new_value) = value.get(component) {
            value = new_value;
        } else {
            // part of path not found abort
            return None;
        }
    }

    return Some(value);
}

// fn format_float(value: f32, round: u8) -> String {
//     // NOTE: this is awful but its the fastest i could do quickly
//     match round {
//         0 => format!("{:.0}", value),
//         1 => format!("{:.1}", value),
//         2 => format!("{:.2}", value),
//         3 => format!("{:.3}", value),
//         4 => format!("{:.4}", value),
//         5 => format!("{:.5}", value),
//         6 => format!("{:.6}", value),
//         7 => format!("{:.7}", value),
//         8 => format!("{:.8}", value),
//         9 => format!("{:.9}", value),
//         _ => panic!("Rounding to {:?} is not supported", round),
//     }

// }

impl Sensor {
    pub fn get_value(&self, sensors: &serde_json::Value) -> Result<String> {
        // if its absolute read the file
        let value = if self.path.is_absolute() {
            std::fs::read_to_string(self.path.as_path())
                .with_context(|| anyhow!("Failed to read path {:?}", self.path))?
        } else {
            // TODO get first part of path and redirect to sensors or some other source
            get_by_path(&sensors, self.path.as_path())
                .map(|x| x.to_string())
                .with_context(|| anyhow!(""))?
        };

        // parse the float
        let mut number: f32 = value
            .parse()
            .with_context(|| anyhow!("Unable to parse float from sensor {:?} output {value:?}", self.name))?;

        // map the value if requested
        if let Some(map) = &self.map {
            number = map.map(number, self.min, self.max);
        }

        // format with specified precision
        let number_str: String = match &self.round {
            None => number.to_string(),
            // NOTE: formats the float with specified number of decimals
            Some(x) => format!("{:.*}", *x as usize, number),
        };

        Ok(number_str)
    }
}

// #[derive(Debug, Clone, Deserialize)]
// enum TemperatureUnit {
//     #[serde(rename = "c")]
//     Celsius,
//
//     #[serde(rename = "f")]
//     Fahrenheit,
// }
//
// impl Default for TemperatureUnit {
//     fn default() -> Self {
//         Self::Celsius
//     }
// }

// TODO implement serialization and default for generating config
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Custom format for output, if not defined all sensors will be shown in a verbose way
    #[serde(default)]
    pub format: Option<String>,

    // /// Use fare
    // pub unit: TemperatureUnit,

    /// How often to poll the temperature in active mode (in millis)
    pub active_poll_rate: u16,

    /// How often to poll the temperature in idle mode (in millis)
    pub idle_poll_rate: u16,

    /// Sensors available in format
    pub sensors: HashMap<String, Sensor>,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     // #[test]
//     // fn test_converter() {
//     //     Sensor::map(…)
//     //     // use SensorConverter::*;
//     //     //
//     //     // let sensor = Sensor {
//     //     //     min: 0.0,
//     //     //     max: 3500.0,
//     //     //     ..Default::default()
//     //     // };
//     //
//     //     assert_eq!(PWM.convert_into(0.0, &sensor), 0.0);
//     //     assert_eq!(PWM.convert_into(3500.0, &sensor), 255.0);
//     //     assert_eq!(PWM.convert_into(1200.0, &sensor), 87.42857);
//     //
//     //     assert_eq!(PROCENTAGE.convert_into(0.0, &sensor), 0.0);
//     //     assert_eq!(PROCENTAGE.convert_into(3500.0, &sensor), 100.0);
//     //     assert_eq!(PROCENTAGE.convert_into(1750.0, &sensor), 50.0);
//     // }
// }

