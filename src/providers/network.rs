// Hegemon - A modular system monitor
// Copyright (C) 2018-2020  Philipp Emanuel Weidmann <pew@worldwidemann.com>
// Copyright (C) 2020       Astro <astro@spaceboyz.net>
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

use std::time::Instant;

use systemstat::{Platform, System};

use crate::stream::{Stream, StreamProvider};

pub struct BandwidthStreamProvider {}

impl StreamProvider for BandwidthStreamProvider {
    fn streams(&self) -> Vec<Box<dyn Stream>> {
        let mut streams = Vec::new();
        let platform = System::new();

        if let Ok(networks) = platform.networks() {
            for network in networks.values() {
                let name = network.name.clone();
                streams.push(Stream::new(
                    format!("{}Rx", name),
                    format!("Ingress bandwidth on {} during the past interval", network.name),
                    rate_calculator(move || {
                        System::new()
                            .network_stats(&name)
                            .ok()
                            .map(|stats| stats.rx_bytes.as_u64() as f64)
                    }),
                    Some(0.0),
                    None,
                    "B",
                    None,
                    1,
                    false,
                ));
                let name = network.name.clone();
                streams.push(Stream::new(
                    format!("{}Tx", name),
                    format!("Egress bandwidth on {} during the past interval", network.name),
                    rate_calculator(move || {
                        System::new()
                            .network_stats(&name)
                            .ok()
                            .map(|stats| stats.tx_bytes.as_u64() as f64)
                    }),
                    Some(0.0),
                    None,
                    "B",
                    None,
                    1,
                    false,
                ));
            }
        }

        streams
    }
}

fn rate_calculator<F>(mut value: F) -> impl FnMut() -> Option<f64> + 'static
where
    F: FnMut() -> Option<f64> + 'static,
{
    let mut last_time = Instant::now();
    let mut last_input = None;
    move || match value() {
        Some(input) => {
            let now = Instant::now();
            let dt = ((now - last_time).as_millis() as f64) / 1000.0;
            let value = last_input.map(|last_input| {
                if input > last_input {
                    (input - last_input) / dt
                } else {
                    0.0
                }
            });
            last_input = Some(input);
            last_time = now;
            value
        }
        None => {
            last_input = None;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandwidth_stream_provider() {
        let streams = BandwidthStreamProvider {}.streams();
        assert!(!streams.is_empty());
    }
}
