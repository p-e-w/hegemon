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

use std::io::{self, Error, ErrorKind};

use systemstat::{CPULoad, DelayedMeasurement, Platform, System};

use stream::{Stream, StreamProvider};

pub struct CPUStreamProvider {}

impl StreamProvider for CPUStreamProvider {
    fn streams(&self) -> Vec<Box<Stream>> {
        let mut streams = Vec::new();

        let mut load: io::Result<DelayedMeasurement<CPULoad>> = Err(Error::new(ErrorKind::Other, ""));

        streams.push(Stream::new(
            "CPU",
            "Average utilization of all CPU cores during the past interval",
            move || {
                let value = if let Ok(ref load) = load {
                    if let Ok(load) = load.done() {
                        Some(f64::from((1.0 - load.idle) * 100.0))
                    } else {
                        None
                    }
                } else {
                    None
                };
                load = System::new().cpu_load_aggregate();
                value
            },
            Some(0.0),
            Some(100.0),
            "%",
            Some(3),
            1,
            false,
        ));

        if let Ok(cpu) = System::new().cpu_load() {
            if let Ok(cpu) = cpu.done() {
                for i in 0..cpu.len() {
                    let mut load: io::Result<DelayedMeasurement<Vec<CPULoad>>> = Err(Error::new(ErrorKind::Other, ""));

                    streams.push(Stream::new(
                        format!("Core{}", i + 1),
                        format!("Utilization of CPU core {} during the past interval", i + 1),
                        move || {
                            let value = if let Ok(ref load) = load {
                                if let Ok(load) = load.done() {
                                    Some(f64::from((1.0 - load[i].idle) * 100.0))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            load = System::new().cpu_load();
                            value
                        },
                        Some(0.0),
                        Some(100.0),
                        "%",
                        Some(3),
                        1,
                        false,
                    ));
                }
            }
        }

        streams
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_stream_provider() {
        let streams = CPUStreamProvider {}.streams();
        assert!(!streams.is_empty());
    }
}
