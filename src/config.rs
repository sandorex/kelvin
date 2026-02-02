use crate::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SensorMap {
    /// Range of values coming from the sensor
    pub input: (f32, f32),

    /// Range of values to map to
    pub output: (f32, f32),
}

impl SensorMap {
    pub fn map(&self, value: f32) -> f32 {
        // clamp the value so it cannot go above or below the limits
        return (
            (value - self.input.0) * (self.output.1 - self.output.0) / (self.input.1 - self.input.0) + self.input.0
        ).clamp(self.output.0, self.output.1)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SensorLabel {
    /// Name to use for the sensor
    pub name: String,

    /// Unit to use after the sensor name
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorSource {
    /// Read a file on filesystem, for exaple sysfs
    File,

    /// Read from lm_sensors output
    Sensors,
}

impl Default for SensorSource {
    fn default() -> Self {
        Self::Sensors
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Sensor {
    /// Name of the sensor
    pub name: String,

    #[serde(default)]
    pub label: Option<SensorLabel>,

    /// Trigger alarm when value goes above the value
    #[serde(default)]
    pub alarm_high: Option<f32>,

    /// Trigger alarm when value falls below the value
    #[serde(default)]
    pub alarm_low: Option<f32>,

    /// How many decimals to round the number to (0 meaning an integer)
    ///
    /// Note that is is only used when the value is shown
    #[serde(default)]
    pub round: Option<u8>,

    /// Map the value into a new range (can be used to convert to/from PWM or percentage)
    #[serde(default)]
    pub map: Option<SensorMap>,

    /// Source of the sensor
    pub source: SensorSource,

    /// Path of the sensor or sensor sysfs file
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

impl Sensor {
    pub fn prefix(&self) -> String {
        // use label name if defined otherwise use name
        format!("{}: ", self.label.as_ref().map(|x| x.name.as_str()).unwrap_or(&self.name))
    }

    pub fn suffix(&self) -> String {
        // use label unit if defined
        format!(" {}", self.label.as_ref().map(|x| x.unit.as_str()).unwrap_or(""))
    }

    /// Get value mapped appropriately
    pub fn get_value(&self, sensors: &serde_json::Value) -> Result<f32> {
        let value = match &self.source {
            SensorSource::File => {
                std::fs::read_to_string(self.path.as_path())
                    .with_context(|| anyhow!("Failed to read path {:?}", self.path))?
                    .trim()
                    .to_string()
            },
            SensorSource::Sensors => {
                get_by_path(&sensors, &self.path)
                    .map(|x| x.to_string())
                    .with_context(|| anyhow!("Unable to find {:?} in lm_sensors output", self.path))?
            }
        };

        let mut number = value
            .parse()
            .with_context(|| anyhow!("Could not parse float from {:?}", value))?;

        // map the value if requested
        if let Some(map) = &self.map {
            number = map.map(number);
        }

        Ok(number)
    }

    /// Returns value formatted properly with the options (rounding, etc)
    pub fn format_value(&self, value: f32) -> String {
        // format with specified precision
        match &self.round {
            None => value.to_string(),
            // NOTE: formats the float with specified number of decimals
            Some(x) => format!("{:.*}", *x as usize, value),
        }
    }
}


// TODO implement serialization and default for generating config
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Custom format for output, if not defined all sensors will be shown in a verbose way
    #[serde(default)]
    pub format: Option<String>,

    // /// Use fare
    // pub unit: TemperatureUnit,

    /// How often to check the temperature (in millis)
    #[serde(default = "Config::default_poll_rate")]
    pub poll_rate: u16,


    /// Sensors available in format
    pub sensors: Vec<Sensor>,
}

/// Get hostname from system using either the environment or `hostname` command
pub fn get_hostname() -> Result<String> {
    // try to get hostname from env var
    if let Ok(env_hostname) = std::env::var("HOSTNAME") {
        return Ok(env_hostname);
    }

    // then as a fallback use hostname executable
    let cmd = std::process::Command::new("hostname")
        .output()
        .with_context(|| "Could not call hostname")?;

    let hostname = String::from_utf8_lossy(&cmd.stdout);

    if !cmd.status.success() || hostname.is_empty() {
        return Err(anyhow!("Unable to get hostname from host"));
    }

    Ok(hostname.trim().into())
}

impl Config {
    fn default_poll_rate() -> u16 {
        crate::MINIMAL_POLL_RATE
    }

    pub fn read_from_file(path: &Path) -> Result<Self> {
        let file_contents = std::fs::read_to_string(path)
            .with_context(|| anyhow!("Unable to read config from file {path:?}"))?;

        let config: Self = toml::from_str(&file_contents)
            .with_context(|| anyhow!("Unable to parse config file {path:?}"))?;

        Ok(config)
    }

    pub fn read_config() -> Result<Self> {
        let hostname = get_hostname()?;

        let config_dir = PathBuf::new()
            .join(std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| "~/.config/".to_string()))
            .join("kelvin");

        let etc_dir = PathBuf::new()
            .join("/etc/kelvin");

        let config_order = vec![
            config_dir.join(format!("{}.toml", hostname)),
            config_dir.join("default.toml"),

            etc_dir.join(format!("{}.toml", hostname)),
            etc_dir.join("default.toml"),
        ];

        for config_file in &config_order {
            if config_file.exists() {
                match Self::read_from_file(config_dir.join(&hostname).as_path()) {
                    Ok(x) => return Ok(x),
                    // print the error so user knows if there are mistakes in the config
                    Err(e) => eprintln!("{}", e),
                }
            }
        }

        bail!("No valid config found in any of following paths\n{config_order:#?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        fn sensor(round: Option<u8>) -> Sensor {
            Sensor {
                round,
                ..Default::default()
            }
        }

        assert_eq!(sensor(None).format_value(7.466321), "7.466321");
        assert_eq!(sensor(Some(3)).format_value(7.466321), "7.466");
        assert_eq!(sensor(Some(2)).format_value(7.466321), "7.47");
        assert_eq!(sensor(Some(0)).format_value(7.466321), "7");
    }

    #[test]
    fn test_value_map() {
        let map = SensorMap { input: (0.0, 1024.0), output: (0.0, 255.0)};
        assert_eq!(map.map(512.0), 127.5);
        assert_eq!(map.map(0.0), 0.0);
        assert_eq!(map.map(1024.0), 255.0);

        // value passed is above or below the limits
        assert_eq!(map.map(-512.0), 0.0);
        assert_eq!(map.map(2000.0), 255.0);
    }
}

