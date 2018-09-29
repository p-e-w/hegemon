// Hegemon - A modular system monitor
// Copyright (C) 2018  Philipp Emanuel Weidmann <pew@worldwidemann.com>
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

use regex::Regex;
use sensors::{FeatureType::SENSORS_FEATURE_FAN, SubfeatureType::SENSORS_SUBFEATURE_FAN_INPUT};

use stream::{StreamProvider, Stream};
use providers::subfeatures;

pub struct FanStreamProvider {
}

impl StreamProvider for FanStreamProvider {
    fn streams(&self) -> Vec<Box<Stream>> {
        let mut streams = Vec::new();

        let regex = Regex::new(r"(?i)fan").unwrap();

        for (subfeature, feature_label, chip_name) in subfeatures(SENSORS_FEATURE_FAN, SENSORS_SUBFEATURE_FAN_INPUT) {
            let name = regex.replace_all(&feature_label, "").replace(" ", "");

            streams.push(Stream::new(
                format!("{}Fan", name),
                format!("Fan speed (feature {} on chip {})", feature_label, chip_name),
                move || {
                    subfeature.get_value().ok()
                },
                None,
                None,
                "RPM",
                Some(4),
                0,
                false,
            ));
        }

        streams
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_fan_stream_provider() {
        let streams = FanStreamProvider{}.streams();
        assert!(!streams.is_empty());
    }
}
