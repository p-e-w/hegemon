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

mod cpu;
mod fan;
mod memory;
mod temperature;

use sensors::{FeatureType, Sensors, Subfeature, SubfeatureType};

use self::cpu::CPUStreamProvider;
use self::fan::FanStreamProvider;
use self::memory::MemoryStreamProvider;
use self::temperature::TemperatureStreamProvider;
use crate::stream::{Stream, StreamProvider};

pub fn streams() -> Vec<Box<dyn Stream>> {
    let providers: Vec<Box<dyn StreamProvider>> = vec![
        Box::new(CPUStreamProvider {}),
        Box::new(MemoryStreamProvider {}),
        Box::new(TemperatureStreamProvider {}),
        Box::new(FanStreamProvider {}),
    ];

    providers.iter().flat_map(|p| p.streams()).collect()
}

fn subfeatures(feature_type: FeatureType, subfeature_type: SubfeatureType) -> Vec<(Subfeature, String, String)> {
    let mut subfeatures = Vec::new();

    for chip in Sensors::new() {
        if let Ok(chip_name) = chip.get_name() {
            for feature in chip {
                if *feature.feature_type() == feature_type {
                    if let Ok(feature_label) = feature.get_label() {
                        for subfeature in feature {
                            if *subfeature.subfeature_type() == subfeature_type {
                                subfeatures.push((subfeature, feature_label.clone(), chip_name.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    subfeatures
}
