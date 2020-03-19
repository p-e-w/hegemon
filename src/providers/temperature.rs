// Hegemon - A modular system monitor
// Copyright (C) 2018-2020  Philipp Emanuel Weidmann <pew@worldwidemann.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use sensors::{FeatureType::SENSORS_FEATURE_TEMP, SubfeatureType::SENSORS_SUBFEATURE_TEMP_INPUT};

use crate::providers::subfeatures;
use crate::stream::{Stream, StreamProvider};

pub struct TemperatureStreamProvider {}

impl StreamProvider for TemperatureStreamProvider {
    fn streams(&self) -> Vec<Box<dyn Stream>> {
        let mut streams = Vec::new();

        let mut core_index = 0;
        let mut package_index = 0;

        for (subfeature, feature_label, chip_name) in subfeatures(SENSORS_FEATURE_TEMP, SENSORS_SUBFEATURE_TEMP_INPUT) {
            let name_description = if feature_label.to_lowercase() == "cpu" {
                Some((String::from("CPU"), String::from("Temperature of CPU")))
            } else if feature_label.to_lowercase().contains("core") {
                core_index += 1;
                Some((
                    format!("Core{}", core_index),
                    format!("Temperature of CPU core {}", core_index),
                ))
            } else if feature_label.to_lowercase().contains("package id") {
                package_index += 1;
                Some((
                    format!("Package {}", package_index),
                    format!("Temperature of CPU package {}", package_index),
                ))
            } else {
                None
            };

            if let Some((name, description)) = name_description {
                streams.push(Stream::new(
                    format!("{}Temp", name),
                    format!("{} (feature {} on chip {})", description, feature_label, chip_name),
                    move || subfeature.get_value().ok(),
                    None,
                    None,
                    "°C",
                    Some(3),
                    1,
                    true,
                ));
            }
        }

        streams
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_temperature_stream_provider() {
        let streams = TemperatureStreamProvider {}.streams();
        assert!(!streams.is_empty());
    }
}
